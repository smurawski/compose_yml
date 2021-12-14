//! Parse a docker-compose.yml file and print it to standard output in
//! normalized format.  Try running:
//!
//! ```sh
//! minicondutor docker-compose.in.yml docker-compose.yml
//! ```

use anyhow::{anyhow, Result};
use compose_yml::v3 as dc;
use regex::Regex;
use std::env;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process;

// Given a build context, ensure that it points to a local directory.
fn git_to_local(ctx: &dc::Context) -> Result<PathBuf> {
    match ctx {
        &dc::Context::GitUrl(ref url) => {
            // Simulate a local checkout of the remote Git repository
            // mentioned in `build`.
            let re = Regex::new(r#"/([^./]+)(?:\.git)?"#).unwrap();
            match re.captures(url.as_ref()) {
                None => Err(anyhow!("Can't get dir name from Git URL: {}", url)),
                Some(caps) => {
                    let path = Path::new(caps.get(1).unwrap().as_str());
                    Ok(path.to_owned())
                }
            }
        }
        &dc::Context::Dir(ref dir) => Ok(dir.clone()),
    }
}

/// Get the local build directory that we'll use for a service.
fn service_build_dir(service: &dc::Service) -> Result<Option<PathBuf>> {
    if let Some(ref build) = service.build {
        let mut path = Path::new("src").to_owned();
        path.push(git_to_local(build.context.value()?)?);
        Ok(Some(path))
    } else {
        Ok(None)
    }
}

/// Update a `docker-compose.yml` file in place.
fn update(file: &mut dc::File) -> Result<()> {
    // Iterate over each name/server pair in the file using `iter_mut`, so
    // we can modify the services.
    for (_name, service) in file.services.iter_mut() {
        // Insert standard env_file entries.
        service.env_files.insert(0, dc::escape("pods/common.env")?);
        service
            .env_files
            .insert(1, dc::raw("pods/overrides/$ENV/common.env")?);

        // Figure out where we'll keep the local checkout, if any.
        let build_dir = service_build_dir(service)?;

        // If we have a local build directory, update the service to use it.
        if let Some(ref dir) = build_dir {
            // Mount the local build directory as `/app` inside the container.
            service
                .volumes
                .push(dc::value(dc::VolumeMount::host(dir, "/app")));
            // Update the `build` field if present.
            if let Some(ref mut build) = service.build {
                build.context = dc::value(dc::Context::Dir(dir.clone()));
            }
        }
    }
    Ok(())
}

/// Our real `main` function.  This is a standard wrapper pattern: we put
/// all the real logic in a function that returns `Result` so that we can
/// use `?` to handle errors, and we reserve `main` just for error
/// handling.
fn run() -> Result<()> {
    // Parse arguments.
    let args: Vec<_> = env::args().collect();
    if args.len() != 3 {
        return Err(anyhow!("Usage: miniconductor <infile> <outfile>"));
    }
    let in_path = Path::new(&args[1]);
    let out_path = Path::new(&args[2]);

    // Transform our file.
    let mut file = dc::File::read_from_path(in_path)?;
    update(&mut file)?;
    file.write_to_path(out_path)?;

    Ok(())
}

fn main() {
    if let Err(ref err) = run() {
        // We use `unwrap` here to turn I/O errors into application panics.
        // If we can't print a message to stderr without an I/O error,
        // the situation is hopeless.
        write!(io::stderr(), "Error: {}", err).unwrap();
        process::exit(1);
    }
}

use std::error::Error;
use std::fs;
use std::io::{StdoutLock, Write};
use std::path::Path;

fn count_lines_in_file(
    file_path: &str,
    count_chars: bool,
) -> Result<(usize, usize), Box<dyn Error>> {
    let contents = fs::read_to_string(file_path)?;
    let line_count = contents.lines().count();

    if count_chars {
        let char_count = contents.chars().count();
        Ok((line_count, char_count))
    } else {
        Ok((line_count, 0))
    }
}

fn count_lines_in_directory(
    path: &Path,
    ignore_files: [&str; 30],
    ignore_folders: [&str; 23],
    total_files: &mut usize,
    total_lines: &mut usize,
    total_chars: &mut usize,
    bad_files: &mut usize,
    handle: &mut StdoutLock,
    log_all_files: bool,
    log_counted_files: bool,
    log_bad_files: bool,
    count_chars: bool,
) -> Result<(), Box<dyn Error>> {
    let path_str = path.to_str().unwrap();
    if ignore_folders
        .iter()
        .any(|folder| path_str.contains(folder))
    {
        return Ok(());
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let file_path = entry.path();
        let file_name = file_path.to_str().unwrap();

        if file_type.is_file()
            && ignore_files.iter().all(|file| !file_name.contains(file))
        {
            *total_files += 1;
            match count_lines_in_file(file_name, count_chars) {
                Ok((line_count, char_count)) => {
                    *total_lines += line_count;
                    if count_chars {
                        *total_chars += char_count;
                    }
                    if log_all_files || log_counted_files {
                        if count_chars {
                            write!(
                            handle,
                            "lines of code in {} = {}, {} chars, total lines = {}\n",
                            file_name, line_count, char_count, total_lines
                        )?;
                        } else {
                            write!(
                                handle,
                                "lines of code in {} = {}, total lines = {}\n",
                                file_name, line_count, total_lines
                            )?;
                        }
                    }
                }
                Err(_) => {
                    *bad_files += 1;
                    if log_all_files || log_bad_files {
                        write!(
                            handle,
                            "Could not count this file: {}\n",
                            file_name
                        )?;
                    }
                }
            }
        } else if file_type.is_dir() {
            count_lines_in_directory(
                &file_path,
                ignore_files,
                ignore_folders,
                total_files,
                total_lines,
                total_chars,
                bad_files,
                handle,
                log_all_files,
                log_counted_files,
                log_bad_files,
                count_chars,
            )?;
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    let project_root = Path::new(".");
    #[rustfmt::skip]
    let ignore_folders = [ ".vscode", "misc", "assets", "android", ".turbo", "dist", "target", ".yarn", "build", ".git", "svg", "icons", "node_modules", ".svelte-kit", ".next", ".solid", ".nuxt", "pocketbase", "images", "fonts", "platforms", "App_Resources", "static", ];
    #[rustfmt::skip]
    let ignore_files = [ ".env", "ignore.json", ".yarnrc.yml", ".prettierignore", "app.d.ts", "todo.txt", "_path.txt", ".eslint.cjs", ".prettierrc", "count.py", ".gitignore", "package-lock.json", "Cargo.lock", "Cargo.toml", "yarn.lock", "pnpm-lock.yaml", "package.json", "tsconfig.json", ".npmrc", "global.d.ts", "svelte.config.js", "tailwind.config.cjs", "postcss.config.cjs", "vite.config.ts", "stats.html", ".eslintcache", "README.md", "TODO.md", ".eslintrc.cjs", ".deepsource.toml", ];

    let mut total_lines = 0;
    let mut total_files = 0;
    let mut total_chars = 0;
    let mut bad_files = 0;
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    let log_all_files = args.len() > 1 && args[1] == "--log-all";
    let log_counted_files = args.len() > 1 && args[1] == "--log-counted";
    let log_bad_files = args.len() > 1 && args[1] == "--log-bad";
    let count_characters = args.iter().any(|x| x == "--chars");

    count_lines_in_directory(
        project_root,
        ignore_files,
        ignore_folders,
        &mut total_files,
        &mut total_lines,
        &mut total_chars,
        &mut bad_files,
        &mut handle,
        log_all_files,
        log_counted_files,
        log_bad_files,
        count_characters,
    )?;

    if count_characters {
        write!(handle, "Counted a total of {} characters\n", total_chars)?;
    }

    write!(
        handle,
        "Across {} files, Counted a total of {} lines, encountered {} BAD FILES\n\nBAD FILES are files that could not be counted, e.g (images, audio, etc..).\n",
        total_files, total_lines, bad_files
    )?;

    Ok(())
}

#![no_std]
#![no_main]

extern crate alloc;

#[macro_use]
extern crate user_lib;

const LF: u8 = 0x0au8;
const CR: u8 = 0x0du8;
const DL: u8 = 0x7fu8;
const BS: u8 = 0x08u8;
const LINE_START: &str = ">> ";

use alloc::string::String;
use alloc::vec::Vec;
use user_lib::console::getchar;
use user_lib::{close, dup, exec, fork, open, pipe, waitpid, OpenFlags};

#[derive(Debug)]
struct ProcessArguments {
    input: String,
    output: String,
    args_copy: Vec<String>,
    args_addr: Vec<*const u8>,
}

impl ProcessArguments {
    pub fn new(command: &str) -> Self {
        let args: Vec<_> = command.split(' ').collect();

        // args all end with '\0'
        let mut args_copy: Vec<String> = args
            .iter()
            .filter(|&arg| !arg.is_empty())
            .map(|&arg| {
                let mut string = String::new();
                string.push_str(arg);
                string.push('\0');
                string
            })
            .collect();

        // redirect input
        // remove '<' and input from args, and store input
        let mut input = String::new();
        if let Some((idx, _)) = args_copy
            .iter()
            .enumerate()
            .find(|(_, arg)| arg.as_str() == "<\0")
        {
            input = args_copy[idx + 1].clone();
            args_copy.drain(idx..=idx + 1);
        }

        // redirect output
        // remove '>' and output from args, and store output
        let mut output = String::new();
        if let Some((idx, _)) = args_copy
            .iter()
            .enumerate()
            .find(|(_, arg)| arg.as_str() == ">\0")
        {
            output = args_copy[idx + 1].clone();
            args_copy.drain(idx..=idx + 1);
        }

        let mut args_addr: Vec<*const u8> = args_copy.iter().map(|arg| arg.as_ptr()).collect();
        args_addr.push(core::ptr::null::<u8>()); // end with 0

        Self {
            input,
            output,
            args_copy,
            args_addr,
        }
    }
}

#[no_mangle]
pub fn main() -> i32 {
    println!("Rust user shell");
    let mut line: String = String::new();
    print!("{}", LINE_START);
    loop {
        let c = getchar();
        // for each character:
        match c {
            // if it is a newline or carriage return:
            // 1. print a newline
            // 2. execute the command in forked process
            // 3. in parent process, wait for the child process to exit
            LF | CR => {
                println!("");
                if !line.is_empty() {
                    let splited: Vec<_> = line.as_str().split('|').collect();
                    let process_arguments_list: Vec<_> = splited
                        .iter()
                        .map(|&cmd| ProcessArguments::new(cmd))
                        .collect();

                    let mut valid = true;
                    // for (i, process_args) in process_arguments_list.iter().enumerate() {
                    //     if i == 0 {
                    //         if !process_args.output.is_empty() {
                    //             valid = false; // first command output should be stdout
                    //         }
                    //     } else if i == process_arguments_list.len() - 1 {
                    //         if !process_args.input.is_empty() {
                    //             valid = false; // last command input should be stdin
                    //         }
                    //     } else if !process_args.output.is_empty() || !process_args.input.is_empty()
                    //     {
                    //         valid = false; // middle command should not have input or output redirection
                    //     }
                    // }

                    // if process_arguments_list.len() == 1 {
                    //     valid = true; // only one command, allow input and output redirection
                    // }

                    // codes in rCore source is stupid. I rewrite it as follow.
                    for (i, process_args) in process_arguments_list.iter().enumerate() {
                        if i > 0 {
                            if !process_args.input.is_empty() {
                                valid = false; // only first command can have input redirection
                            }
                        }
                        if i < process_arguments_list.len() - 1 {
                            if !process_args.output.is_empty() {
                                valid = false; // only last command can have output redirection
                            }
                        }
                    }

                    if !valid {
                        println!("Invalid command: Inputs/Outputs cannot be correctly binded!");
                    } else {
                        // command valid, execute

                        // for n commands, n - 1 pipes are needed
                        let mut pipes_fd: Vec<[usize; 2]> = Vec::new();
                        if !process_arguments_list.is_empty() {
                            for _ in 0..process_arguments_list.len() - 1 {
                                let mut pipe_fd = [0usize; 2];
                                pipe(&mut pipe_fd);
                                pipes_fd.push(pipe_fd);
                            }
                        }

                        // execute each process
                        let mut children: Vec<_> = Vec::new();
                        for (i, process_argument) in process_arguments_list.iter().enumerate() {
                            let pid = fork();
                            if pid == 0 {
                                let input = &process_argument.input;
                                let output = &process_argument.output;
                                let args_copy = &process_argument.args_copy;
                                let args_addr = &process_argument.args_addr;

                                // redirect non-pipe input
                                if !input.is_empty() {
                                    let input_fd = open(input.as_str(), OpenFlags::RDONLY);
                                    if input_fd == -1 {
                                        println!("Error when opening file {}", input);
                                        return -4;
                                    }
                                    let input_fd = input_fd as usize;
                                    // close 0, dup as 0, close old fd
                                    // redirect stdin to input file
                                    close(0);
                                    assert_eq!(dup(input_fd), 0);
                                    close(input_fd);
                                }

                                // redirect non-pipe output
                                if !output.is_empty() {
                                    let output_fd = open(
                                        output.as_str(),
                                        OpenFlags::CREATE | OpenFlags::WRONLY,
                                    );
                                    if output_fd == -1 {
                                        println!("Error when opening file {}", output);
                                        return -4;
                                    }
                                    let output_fd = output_fd as usize;
                                    // close 1, dup as 1, close old fd
                                    // redirect stdout to output file
                                    close(1);
                                    assert_eq!(dup(output_fd), 1);
                                    close(output_fd);
                                }

                                // receive input from the previous process
                                if i > 0 {
                                    // close 0, dup pipe read as 0
                                    // will close pipe read later
                                    close(0);
                                    let read_end = pipes_fd.get(i - 1).unwrap()[0];
                                    assert_eq!(dup(read_end), 0);
                                }

                                // send output to the next process
                                if i < process_arguments_list.len() - 1 {
                                    // close 1, dup pipe write as 1
                                    // will close pipe write later
                                    close(1);
                                    let write_end = pipes_fd.get(i).unwrap()[1];
                                    assert_eq!(dup(write_end), 1);
                                }

                                // close all pipe ends inherited from the parent process
                                for pipe_fd in pipes_fd.iter() {
                                    close(pipe_fd[0]);
                                    close(pipe_fd[1]);
                                }

                                // execute new application
                                if exec(args_copy[0].as_str(), args_addr.as_slice()) == -1 {
                                    println!("Error when executing!");
                                    return -4;
                                }
                                unreachable!();
                            } else {
                                children.push(pid); // main process, store child pid
                            }
                        }

                        // main process, close all pipes, we do not need any more
                        for pipe_fd in pipes_fd.iter() {
                            close(pipe_fd[0]);
                            close(pipe_fd[1]);
                        }

                        // main process, wait for all child process
                        // and also release the resource of child process
                        let mut exit_code: i32 = 0;
                        for pid in children.into_iter() {
                            let exit_pid = waitpid(pid as usize, &mut exit_code);
                            assert_eq!(pid, exit_pid);
                            println!("Shell: Process {} exited with code {}", pid, exit_code);
                        }
                    }

                    line.clear();
                }
                // finally, print prompt
                print!("{}", LINE_START);
            }
            // if it is a backspace or delete:
            BS | DL => {
                if !line.is_empty() {
                    // backward the cursor
                    print!("{}", BS as char);
                    // print a space, overwrite the character
                    print!(" ");
                    // backward the cursor again, now the cursor is at the position of the character to be deleted
                    print!("{}", BS as char);
                    // remove the last character
                    line.pop();
                }
            }
            _ => {
                print!("{}", c as char);
                line.push(c as char);
            }
        }
    }
}

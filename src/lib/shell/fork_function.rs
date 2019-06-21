use super::{fork::IonResult, sys, variables::Value, Capture, Shell};
use std::process;

impl<'a> Shell<'a> {
    /// High-level function for executing a function programmatically.
    /// NOTE: Always add "ion" as a first argument in `args`.
    pub fn fork_function<S: AsRef<str>, T, F: FnOnce(IonResult) -> Result<T, ()>>(
        &self,
        capture: Capture,
        result: F,
        fn_name: &str,
        args: &[S],
    ) -> Result<T, ()> {
        if let Some(Value::Function(function)) = self.variables.get(fn_name) {
            let output = self
                .fork(capture, move |child| {
                    if let Err(err) = function.execute(child, args) {
                        if capture == Capture::None {
                            eprintln!("ion: {} function call: {}", fn_name, err);
                        }
                    }
                    Ok(())
                })
                .map_err(|err| eprintln!("ion: fork error: {}", err))
                .and_then(result);

            // Ensure that the parent retains ownership of the terminal before exiting.
            let _ = sys::tcsetpgrp(libc::STDIN_FILENO, process::id());
            output
        } else {
            Err(())
        }
    }

    /// Execute the function on command not found
    pub fn command_not_found<S: AsRef<str>>(&self, cmd: S) {
        if self
            .fork_function(Capture::None, |_| Ok(()), "COMMAND_NOT_FOUND", &["ion", cmd.as_ref()])
            .is_err()
        {
            eprintln!("ion: command not found: {}", cmd.as_ref());
        }
    }
}

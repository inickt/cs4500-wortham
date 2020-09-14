use std::io::Write;
use std::time::{ Instant, Duration };

const LIMIT_ARG: &str = "-limit";
const LIMITED_NUM_LINES: i32 = 20;
const DEFAULT_OUTPUT: &str = "hello world";
const ARG_SEPARATOR: &str = " ";

fn main() {
    let args = std::env::args().skip(1).collect();
    let stdout = std::io::stdout();
    yes(args, stdout.lock(), None);
}

// Given the command-line arguments (excluding the program name)
// concatenate them all with a " " and write them to the output buffer forever.
// If -limit is the first argument, exclude it from the output, and only print 20 lines.
// stop_after is used in testing to stop looping after a given time rather than iteration count
// to test the case where no "-limit" option is given.
fn yes<W: Write>(args: Vec<String>, mut output_buffer: W, stop_after: Option<Duration>) {
    let (mut limit, args) = if args.get(0) == Some(&(LIMIT_ARG.to_string())) { 
        (Some(LIMITED_NUM_LINES), &args[1..])
    } else {
        (None, &args[..])
    };

    let output = if args.is_empty() {
        DEFAULT_OUTPUT.to_string()
    } else {
        args.join(ARG_SEPARATOR)
    };

    let start = Instant::now();

    // if LIMIT_ARG was not passed (limit is None), print output forever
    // otherwise, print output LIMITED_NUM_LINES times
    while limit != Some(0) && stop_after.map_or(true, |duration| start.elapsed() <= duration) {
        writeln!(output_buffer, "{}", output).expect("Error writing to output");
        limit = limit.map(|x| x - 1);
    }
}

#[test]
fn test_limit() -> Result<(), Box<dyn std::error::Error>> {
    let tests = &[
        (vec!["-limit"], "hello world\n".repeat(20)),
        (vec!["-limit", "hi", "there", "how's", "it", "going?"], "hi there how's it going?\n".repeat(20)),
        (vec!["-limit", ""], "\n".repeat(20)),
    ];

    for (args, expected_output) in tests {
        let mut output = vec![];
        let args = args.iter().map(|arg| arg.to_string()).collect();

        yes(args, &mut output, None);

        let stdout = String::from_utf8(output)?;
        assert_eq!(stdout, *expected_output);
    }
    Ok(())
}

#[test]
fn test_unlimited() -> Result<(), Box<dyn std::error::Error>> {
    let tests = &[
        (vec![], "hello world"),
        (vec!["hi", "there", "how's", "it", "going?"], "hi there how's it going?"),
        (vec![""], ""),
    ];

    // We cannot test all infinite lines of output so we limit it to running for 5ms, 10ms, and 100ms.
    // For each run, we assert every line printed is equal to the expected_output line from the
    // tests array above. Additionally, we assert that the array of test_outputs containing the
    // output of each run is in sorted order by number of lines. That is, that the 10ms run printed
    // more lines than the 5ms run and the 100ms run printed more lines than the 10ms run. We don't
    // assume the number of lines scales perfectly linearly since this can be affected by outside
    // factors like the kernel's task scheduling.
    let times = vec![
        Duration::from_millis(5),
        Duration::from_millis(10),
        Duration::from_millis(100),
    ];

    for (args, expected_output) in tests {
        let args: Vec<String> = args.iter().map(|arg| arg.to_string()).collect();

        let line_counts: Vec<usize> = times.iter().copied().map(|duration| {
            let mut buffer = vec![];
            yes(args.clone(), &mut buffer, Some(duration));

            let output = String::from_utf8(buffer).expect("output contains invalid utf8");
            assert!(output.lines().all(|line| line == *expected_output));
            output.len()
        }).collect();

        // Assert given xyes runs for more time it outputs more lines each time.
        // Alas, we cannot test all possible durations so we limit it to 5ms, 10ms, and 100ms.
        let mut sorted_line_counts = line_counts.clone();
        sorted_line_counts.sort();
        assert_eq!(line_counts, sorted_line_counts);
    }
    Ok(())
}

const LIMIT_ARG: &str = "-limit";
const LIMITED_NUM_LINES: i32 = 20;
const DEFAULT_OUTPUT: &str = "hello world";
const ARG_SEPARATOR: &str = " ";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (mut limit, args) = if args.get(1) == Some(&(LIMIT_ARG.to_string())) { 
        (Some(LIMITED_NUM_LINES), &args[2..])
    } else {
        (None, &args[1..])
    };

    let output = if args.is_empty() {
        DEFAULT_OUTPUT.to_string()
    } else {
        args.join(ARG_SEPARATOR)
    };

    // if LIMIT_ARG was not passed (limit is None), print output forever
    // otherwise, print output LIMITED_NUM_LINES times
    while limit != Some(0) {
        println!("{}", output);
        limit = limit.map(|x| x - 1);
    }
}

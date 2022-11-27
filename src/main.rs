mod cmd;

use cmd::parser::parse_cli_args;


fn main() {
    let cli_args = parse_cli_args();
    println!("{:?}", cli_args);
    let config = cli_args.read_config();
    println!("{:?}", config);
    println!("{:?}", config.read_apis());
}

use std::os;
use optparse::{Opt, OptionParser};

mod optparse;

fn main() {
	// first, define the options that the command supports
	let simple_opt = Opt::new("-o", "--opt", "A simple option");
	let opt_with_opt_arg = Opt::new("-a", "--opt-with-arg [ARG]", "An option taking an optional argument");
	let opt_with_req_arg = Opt::new("-r", "--required-arg ARG", "An option taking a required argument");
	let long_opt = Opt::new("", "--long-opt", "An option with no short variant");
	let int_arg = Opt::new("-i", "--int-arg [ARG]", "Option that takes an int arg");
	let multi_value_arg = Opt::new("-m", "--multi-arg [ARGS]", "Option that can be repeated");
	let version_opt = Opt::version_opt();

	// specify the syntax, banner and options for the command
	let opt_parser = OptionParser {
		usage : "[<values to print>...]".to_owned(),
		banner : "This is an example app for the optparser module. \
		           The banner is a short summary which appears at the top of
		           --help output".to_owned(),
		opts : ~[&simple_opt, &opt_with_opt_arg, &long_opt, &int_arg, &opt_with_req_arg,
		         &multi_value_arg, &version_opt],
		tail_banner : Some("This is a tail banner that appears below the list of options".to_owned())
	};

	let flags = opt_parser.parse(os::args());

	match flags.status {
		optparse::Help => return,
		optparse::Error => {
			os::set_exit_status(1);
			return
		},
		_ => ()
	}

	// handle options
	if opt_parser.is_set(&flags, &simple_opt) {
		println!("An option with no args was used");
	}

	opt_parser.with_value(&flags, &opt_with_opt_arg, |val| {
		println!("An option with optional arg {} was used", val);
	});

	opt_parser.with_value(&flags, &opt_with_req_arg, |val| {
		println!("An option with required arg {} was used", val);
	});

	if opt_parser.is_set(&flags, &long_opt) {
		println!("An option with only the long opt form was used");
	}

	if opt_parser.is_set(&flags, &version_opt) {
		println!("Version opt was used");
	}

	match opt_parser.value(&flags, &opt_with_opt_arg) {
		Some(v) => println!("An option with an optional arg {} was used", v),
		None => ()
	}

	let multi_opt_values = opt_parser.values(&flags, &multi_value_arg);
	for val in multi_opt_values.iter() {
		println!("Multi-value arg: {}", *val);
	}

	opt_parser.with_value(&flags, &int_arg, |val| {
		let int_val : Option<int> = from_str(val);
		match int_val {
			Some(int_val) =>
				println!("An option which expects an int arg was used: {}", int_val),
			None =>
				println!("{} expects a numeric arg", int_arg.long)
		}
	});

	// handle remaining args
	for (i, arg) in flags.args.iter().enumerate() {
		println!("Non-option argument {}: {}", i, *arg);
	}
}

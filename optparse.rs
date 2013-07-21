extern mod extra;
extern mod std;

use std::os;

/// Represents a command-line flag
pub struct Opt {
	/// Short version of argument, consisting of a '-' followed
	/// by a single letter
	short: ~str,
	/// Long version of argument, consisting of '--' followed by
	/// one or more letters
	long: ~str,
	/// A brief description of the option
	description: ~str
}

/// Parser for processing command-line arguments and displaying
/// usage information
pub struct OptionParser<'self> {
	/// A one line usage summary to be displayed by -h.
	/// The output format is '<program name> <usage>'
	usage: ~str,
	/// A short summary of what the command does,
	/// displayed underneath the 'usage' string by -h
	/// and before the list of options
	banner: ~str,
	/// Vector of accepted option flags
	opts: ~[&'self Opt],
	/// A banner that is displayed below the list
	/// of options
	tail_banner : Option<~str>
}

struct OptMatch {
	opt_name : ~str,
	val : ~str
}

/// Enum indicating whether a set of command-line arguments
/// were parsed successfully
pub enum ParseStatus {
	/// The command line arguments were parsed successfully
	Success,
	/// An option to print usage information (eg. --help)
	/// was used
	Help,
	/// An error occurred whilst parsing the arguments
	Error
}

/// Holds the result of a call to OptionParser::parse(),
/// storing information about matching command-line flags and
/// the list of non-flag arguments on the command-line
pub struct ParseResult {
	opts : ~[OptMatch],
	status : ParseStatus,
	args : ~[~str]
}

// word-wraps a string to fit 'cols' columns.  Lines start at column
// 'start_col'
fn word_wrap_str(s: &str, start_col : uint, cols : uint) -> ~str {
	let mut wrapped = ~"";
	let mut line_spaces_left = cols - start_col;
	let mut first_in_line = true;

	for s.word_iter().advance() |word| {
		if line_spaces_left < word.len() {
			wrapped.push_char('\n');
			for std::uint::range(0, start_col) |_| {
				wrapped.push_char(' ');
			}
			line_spaces_left = cols - start_col;
			first_in_line = true;
		} else {
			line_spaces_left -= word.len();
		}
		if first_in_line {
			first_in_line = false;
		} else {
			wrapped.push_char(' ');
			line_spaces_left -= 1;
		}
		wrapped.push_str(word);
	}

	wrapped
}

impl Opt {
	/// Returns true if a given argument string
	/// (either in the form '-o' or '--option') matches
	/// this option.
	fn match_arg(&self, arg : &str) -> bool {
		arg == self.short || arg == self.long_parsed()
	}

	/// Returns the long form of this option, minus any
	/// argument (eg. if self.long == '--option [arg]', this
	/// returns '--option')
	fn long_parsed<'r>(&'r self) -> &'r str {
		self.long.split_iter(' ').take_(1).next().get()
	}

	/// Constructs a new option with the given syntax.
	///
	/// @p short is the single-letter variant, starting with
	/// a single dash, eg. '-o'.  This may be an empty string.
	///
	/// @p long is the multi-letter variant, starting with double-dashes
	/// and optionally including an argument name, eg. '--option' or
	/// '--option [arg]'.  This may not be an empty string.
	///
	/// @p description Is a brief description of the option for use
	/// in --help output
	pub fn new(short: &str, long: &str, description: &str) -> Opt {
		Opt {
			short : short.to_owned(),
			long : long.to_owned(),
			description : description.to_owned()
		}
	}

	/// Returns an Opt struct for the '--help' option
	fn help_opt() -> Opt {
		Opt {
			short : ~"-h",
			long : ~"--help",
			description : ~"Print usage information"
		}
	}

	/// Returns true if this option takes an argument
	pub fn has_arg(&self) -> bool {
		self.long.contains(" ")
	}

	/// Returns true if this option takes a mandatory argument
	pub fn has_required_arg(&self) -> bool {
		self.has_arg() && !self.long.contains("[")
	}
}

impl <'self> OptionParser<'self> {
	// iterates over each flag in a command-line argument
	fn each_opt_in_arg(arg : &str, callback : &fn(&str)) {
		if arg.starts_with("--") {
			callback(arg);
		} else if (arg.starts_with("-")) {
			for arg.slice_from(1).iter().advance() |c| {
				callback(fmt!("-%c", c));
			}
		}
	}

	/// Parse a list of command-line arguments,
	/// print out usage information if --help was
	/// specified or errors if the arguments are incorrect
	/// and return a ParseResult indicating the options
	/// that were set.
	///
	/// Successfully parsed options can be retrieved using
	/// is_set(), value() or with_value() on the result.
	pub fn parse(&self, args: ~[~str]) -> ParseResult {
		let mut result = ParseResult {
			opts : ~[],
			status : Success,
			args : ~[]
		};

		let mut had_error = false;
		let mut skip_next_arg = false;
		for args.iter().enumerate().advance |(index, arg)| {
			if skip_next_arg {
				skip_next_arg = false;
				loop
			}

			let mut is_opt = false;
			do OptionParser::each_opt_in_arg(*arg) |opt_arg| {
				is_opt = true;
				let matching_opt = do self.opts.iter().find_ |opt| {
					opt.match_arg(opt_arg)
				};
				match matching_opt {
					Some(opt) => {
						let has_arg =
						  opt.has_arg() &&
						  index < args.len()-1 &&
						  (arg.starts_with("--") || arg.len() == 2);
						if has_arg {
							skip_next_arg = true;
							result.opts.push(OptMatch {
								opt_name : opt.long_parsed().to_owned(),
								val : args[index+1].clone()
							});
						} else {
							if opt.has_required_arg() {
								if !had_error {
									println(fmt!("Missing required argument for option %s.\n\n%s\n", opt_arg, OptionParser::arg_help_str(*opt)));
									had_error = true;
								}
							} else {
								result.opts.push(OptMatch {
									opt_name : opt.long_parsed().to_owned(),
									val : ~""
								});
							}
						};
					},
					None => {
						let help_opt = Opt::help_opt();
						if help_opt.match_arg(opt_arg) {
							self.print_usage();
							result.status = Help;
						} else {
							if !had_error {
								match self.suggest_opt(*arg) {
									Some(opt) => {
										println(fmt!("Unknown option %s, did you mean '%s'?\n\n%s\n",
										  opt_arg,
										  opt.long_parsed(),
										  OptionParser::arg_help_str(opt)))
									}
									None => {
										println(fmt!("Unknown option %s", opt_arg));
									}
								}
								had_error = true;
							}
						}
					}
				}
			}
			
			if !is_opt && index > 0 {
				result.args.push(copy *arg);
			}
		}

		if had_error {
			result.status = Error;
		}
		result
	}

	/// Prints usage information for the command-line options.
	/// This has the same effect as passing the -h flag
	fn print_usage(&self) {
		println(self.format_help_str());
	}

	fn arg_help_str(opt: &Opt) -> ~str {
		let mut help_str = if opt.short.len() > 0 {
			fmt!("  %s, %s", opt.short, opt.long)
		} else {
			fmt!("      %s", opt.long)
		};
		
		let DESCRIPTION_COL = 26;
		let first_line_len;

		if help_str.len() < DESCRIPTION_COL {
			first_line_len = help_str.len();
		} else {
			help_str.push_char('\n');
			first_line_len = 0;
		}

		for std::uint::range(first_line_len, DESCRIPTION_COL) |_| {
			help_str.push_char(' ');
		}

		help_str + word_wrap_str(opt.description, DESCRIPTION_COL, 80)
	}

	fn format_help_str(&self) -> ~str {
		let usage_str : &str = fmt!("Usage: %s %s", os::args()[0], self.usage);

		struct OptHelpEntry<'self> {
			help_str : ~str,
			sort_key : &'self str
		};

		let mut opt_list = self.opts.map(|opt| {
			OptHelpEntry {
				help_str : OptionParser::arg_help_str(*opt),
				sort_key : opt.long
			}
		});
		extra::sort::quick_sort(opt_list, |a,b| {
			a.sort_key < b.sort_key
		});

		let banner : &str = word_wrap_str(self.banner, 0, 80);
		let opt_section : &str = opt_list.map(|entry| {
			entry.help_str.clone()
		}).connect("\n");
		let mut sections = ~[usage_str, banner, opt_section];

		match self.tail_banner {
			Some(ref _tail) => {
				let tail : &str = *_tail;
				sections.push(tail)
			}
			None() => ()
		}

		sections.connect("\n\n").append("\n")
	}

	// for a given input argument string, returns the registered
	// option with the closest spelling
	fn suggest_opt<'a>(&'a self, input : &str) -> Option<&'a Opt> {
		let mut min_edit_dist = std::uint::max_value;
		let mut suggested_opt : Option<&'a Opt> = None;
		for self.opts.iter().advance |opt| {
			let edit_dist = opt.long_parsed().lev_distance(input);
			if edit_dist < min_edit_dist {
				min_edit_dist = edit_dist;
				suggested_opt = Some(*opt);
			}
		}
		suggested_opt
	}

	fn is_valid_opt(&self, name : &str) -> bool {
		let opt = do self.opts.iter().find_ |opt| {
			let opt_name : &str = opt.long_parsed();
			opt_name == name
		};
		opt.is_some()
	}

	/// Invokes action() with the value of a given option if it was set
	pub fn with_value(&self, flags : &ParseResult, opt: &Opt, action : &fn(&str)) {
		match self.value(flags, opt) {
			Some(value) => action(value),
			None => ()
		}
	}

	/// Returns the value for a given option if set or None otherwise
	pub fn value<'r>(&self, flags : &'r ParseResult, match_opt: &Opt) -> Option<&'r str> {
		match flags.opts.iter().find_(|opt| {
			let name : &str = opt.opt_name;
			name == match_opt.long_parsed()
		}) {
			Some(match_) => {
				let opt_value : &'r str = match_.val;
				Some(opt_value)
			},
			None => None
		}
	}

	/// Returns true if a given flag was passed on the command-line
	pub fn is_set(&self, flags : &ParseResult, opt: &Opt) -> bool {
		match self.value(flags, opt) {
			Some(_) => true,
			None => false
		}
	}
}



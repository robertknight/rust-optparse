use std::os;
use std::io::stdio::print;
use std::num::Bounded;

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
pub struct OptionParser<'a> {
	/// A one line usage summary to be displayed by -h.
	/// The output format is '<program name> <usage>'
	usage: ~str,
	/// A short summary of what the command does,
	/// displayed underneath the 'usage' string by -h
	/// and before the list of options
	banner: ~str,
	/// Vector of accepted option flags
	opts: ~[&'a Opt],
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
	opts : Vec<OptMatch>,
	status : ParseStatus,
	args : Vec<~str>
}

// word-wraps a string to fit 'cols' columns.  Lines start at column
// 'start_col'
fn word_wrap_str(s: &str, start_col : uint, cols : uint) -> ~str {
	let mut wrapped = StrBuf::new();
	let mut line_spaces_left = cols - start_col;
	let mut first_in_line = true;

	for word in s.words() {
		if line_spaces_left < word.len() {
			wrapped.push_char('\n');
			for _ in range(0, start_col) {
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

	wrapped.into_owned()
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
		self.long.split(' ').take(1).next().unwrap()
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
	pub fn help_opt() -> Opt {
		Opt {
			short : ~"-h",
			long : ~"--help",
			description : ~"Display usage information"
		}
	}

	/// Returns an Opt struct for a --version flag
	pub fn version_opt() -> Opt {
		Opt {
			short : ~"-v",
			long : ~"--version",
			description : ~"Display version information"
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

impl <'a> OptionParser<'a> {

	/// Creates a new OptionParser
	pub fn new<'a>(usage:&str, banner:&str, opts:&[&'a Opt]) -> OptionParser<'a> {
		OptionParser {
			usage : usage.to_owned(),
			banner : banner.to_owned(),
			opts : opts.to_owned(),
			tail_banner : None
		}
	}

	// iterates over each flag in a command-line argument
	fn each_opt_in_arg(arg : &str, f : &fn(&str)) {
		if arg.starts_with("--") {
			f(arg);
		} else if (arg.starts_with("-")) {
			for c in arg.slice_from(1).chars() {
				f(format!("-{}", c));
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
			opts : Vec::new(),
			status : Success,
			args : Vec::new()
		};

		let mut opts : Vec<&Opt> = vec!();
		for opt in self.opts.iter() {
			opts.push(*opt);
		}
		let help_opt = Opt::help_opt();
		opts.push(&help_opt);

		let mut had_error = false;
		let mut skip_next_arg = false;
		for (index, arg) in args.iter().enumerate() {
			if skip_next_arg {
				skip_next_arg = false;
				continue
			}

			let mut is_opt = false;
			for opt_arg in OptionParser::each_opt_in_arg(*arg) {
				is_opt = true;
				let matching_opt = for opt in opts.iter().find_ {
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
									println!("Missing required argument for option {}.\n\n{}\n", opt_arg, OptionParser::arg_help_str(*opt));
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
						if !had_error {
							match self.suggest_opt(*arg) {
								Some(opt) => {
									println!("Unknown option {}, did you mean '{}'?\n\n{}\n",
									  opt_arg,
									  opt.long_parsed(),
									  OptionParser::arg_help_str(opt))
								}
								None => {
									println!("Unknown option {}", opt_arg);
								}
							}
							had_error = true;
						}
					}
				}
			}

			if !is_opt && index > 0 {
				result.args.push(arg.clone());
			}
		}

		if had_error {
			result.status = Error;
		} else {
			// handle built-in options
			if self.is_set(&result, &help_opt) {
				self.print_usage();
				result.status = Help;
			}
		}

		result
	}

	/// Prints usage information for the command-line options.
	/// This has the same effect as passing the -h flag
	pub fn print_usage(&self) {
		print(self.format_help_str());
	}

	fn arg_help_str(opt: &Opt) -> ~str {
		let mut help_str = if opt.short.len() > 0 {
			format!("  {}, {}", opt.short, opt.long)
		} else {
			format!("      {}", opt.long)
		};

		let DESCRIPTION_COL = 26;
		let first_line_len;

		if help_str.len() < DESCRIPTION_COL {
			first_line_len = help_str.len();
		} else {
			help_str.push_char('\n');
			first_line_len = 0;
		}

		for _ in range(first_line_len, DESCRIPTION_COL) {
			help_str.push_char(' ');
		}

		help_str + word_wrap_str(opt.description, DESCRIPTION_COL, 80)
	}

	/// Returns a string containing the --help output
	/// for the current set of arguments
	pub fn format_help_str(&self) -> ~str {
		let usage_str : &str = format!("Usage: {} {}", os::args()[0], self.usage);

		struct OptHelpEntry<'a> {
			help_str : ~str,
			sort_key : &'a str
		};

		let mut opt_list = self.opts.map(|opt| {
			OptHelpEntry {
				help_str : OptionParser::arg_help_str(*opt),
				sort_key : opt.long
			}
		});
		opt_list.sort_by(|a,b| {
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
			None => ()
		}

		sections.connect("\n\n").append("\n")
	}

	// for a given input argument string, returns the registered
	// option with the closest spelling
	fn suggest_opt<'a>(&'a self, input : &str) -> Option<&'a Opt> {
		let mut min_edit_dist : uint = Bounded::max_value();
		let mut suggested_opt : Option<&'a Opt> = None;
		for opt in self.opts.iter() {
			let edit_dist = opt.long_parsed().lev_distance(input);
			if edit_dist < min_edit_dist {
				min_edit_dist = edit_dist;
				suggested_opt = Some(*opt);
			}
		}
		suggested_opt
	}

	fn is_valid_opt(&self, name : &str) -> bool {
		let opt = for opt in self.opts.iter().find_ {
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
		let matches = self.values(flags, match_opt);
		if matches.len() > 0 {
			Some(matches[0])
		} else {
			None
		}
	}

	/// Returns all of the values for a given option
	pub fn values<'r>(&self, flags : &'r ParseResult, match_opt: &Opt) -> ~[&'r str] {
		let mut matches = ~[];
		for opt_match in flags.opts.iter() {
			let name : &str = opt_match.opt_name;
			if name == match_opt.long_parsed() {
				let val : &'r str = opt_match.val;
				matches.push(val);
			}
		}
		matches
	}

	/// Returns true if a given flag was passed on the command-line
	pub fn is_set(&self, flags : &ParseResult, opt: &Opt) -> bool {
		match self.value(flags, opt) {
			Some(_) => true,
			None => false
		}
	}
}

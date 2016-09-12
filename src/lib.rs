/// Library to parse and execute brainfuck programs

use std::error::Error;
use std::fmt;
use std::io::stdin;

pub use self::Command::*;
pub use self::Ast::*;

/// Brainfuck command
#[derive(Debug)]
pub enum Command {
    IncPointer,
    DecPointer,
    IncData,
    DecData,
    GetByte,
    PutByte,
}

/// Node in a brainfuck AST
#[derive(Debug)]
pub enum Ast {
    Op(Command),
    Loop(Program),
}

/// A brainfuck program is just a list of AST nodes
pub type Program = Vec<Ast>;

/// Context in which a brainfuck program executes
#[derive(Debug)]
pub struct Context {
    dp: usize,
    data: Vec<u8>,
}

impl Context {
    /// build a new program context initialized with all zeroes
    pub fn new() -> Context {
        Context { dp: 0, data: Vec::with_capacity(100) }
    }

    /// execute program `p` in this context
    pub fn execute(&mut self, p: &Program) -> Result<(), String> {
        for node in p {
            let cur_data = self.cur_data();
            match *node {
                Op(IncPointer) => self.dp = self.dp.wrapping_add(1),
                Op(DecPointer) => self.dp = self.dp.wrapping_sub(1),
                Op(IncData)    => self.set_cur_data(cur_data.wrapping_add(1)),
                Op(DecData)    => self.set_cur_data(cur_data.wrapping_sub(1)),
                Op(GetByte)    => self.set_cur_data(try!(Context::getbyte())),
                Op(PutByte)    => print!("{:}", self.cur_data() as char),
                Loop(ref x)    =>  {
                    while self.getdata(self.dp) != 0 {
                        try!(self.execute(x))
                    }
                },
            }
        }
        Ok(())
    }

    /// set data cell at `address` to `value`
    ///
    /// It is preferred that you use this rather than accessing the data cell
    /// directly, as this will ensure the address is in fact allocated,
    /// preventing panics
    pub fn setdata(&mut self, address: usize, value: u8) {
        if address >= self.data.len() {
            let diff = address - self.data.len() + 1;
            self.data.extend((0..diff).map(|_| 0u8));

            // after we extend, address should be within bounds
            assert!(address < self.data.len());
        };
        self.data[address] = value;
    }

    /// get data cell at `address`
    ///
    /// It is preferred that you use this rather than accessing the data cell
    /// directly, as this will ensure the address is in fact allocated,
    /// preventing panics
    pub fn getdata(&self, address: usize) -> u8 {
        if address >= self.data.len() { 0 } else { self.data[address] }
    }

    fn cur_data(&self) -> u8 {
        self.getdata(self.dp)
    }

    fn set_cur_data(&mut self, value: u8) {
        let dp = self.dp;
        self.setdata(dp, value)
    }

    fn getbyte() -> Result<u8, String> {
        loop {
            let mut input = String::new();
            println!("enter single byte: ");
            try!(stdin().read_line(&mut input)
                .map_err(|e| format!("could not read char: {}", e)));

            if input.len() > 1 {
                println!("only a single char, please");
                continue;
            } else if (input.chars().next().unwrap() as u32) > 256 {
                println!("char must fit in a single byte");
                continue;
            } else {
                return Ok(input.as_bytes()[0]);
            }
        }
    }
}

/// Used for parsing errors
#[derive(Debug)]
pub struct ParseError {
    description: String
}

impl ParseError {
    fn new(d: &str) -> ParseError {
        ParseError { description: d.to_string() }
    }
}

impl Error for ParseError {
    fn description(&self) -> &str { &self.description }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "ParseError: {}", self.description)
    }
}

enum ParseResult {
    Some(Ast),
    Ignore,
    LoopEnd,
    Err(ParseError),
}

/// parse a brainfuck program
///
/// # Failures
/// There's not many texts that aren't valid as a brainfuck program. This
/// function will only error in the case of unbalanced '[]' tokens
pub fn parse<T>(stream: &mut T) -> Result<Program, ParseError>
    where T: Iterator<Item=char>
{
    let mut program: Program = Vec::with_capacity(20);
    while let Some(c) = stream.next() {
        use self::ParseResult::*;
        match parse_char(c, stream) {
            Some(ast) => program.push(ast),
            Ignore => continue,
            LoopEnd => return Result::Err(ParseError::new("extra ']'")),
            Err(x) => return Result::Err(x)
        }
    }
    Ok(program)
}

fn parse_char<T>(c: char, stream: &mut T) -> ParseResult
    where T: Iterator<Item=char>
{
    use self::ParseResult::*;
    match c {
        '>' => Some(Op(IncPointer)),
        '<' => Some(Op(DecPointer)),
        '+' => Some(Op(IncData)),
        '-' => Some(Op(DecData)),
        ',' => Some(Op(GetByte)),
        '.' => Some(Op(PutByte)),
        '[' => parse_loop(stream),
        ']' => LoopEnd,
        _   => Ignore,
    }
}

fn parse_loop<T>(stream: &mut T) -> ParseResult
    where T: Iterator<Item=char>
{
    let mut commands: Program = Vec::with_capacity(20);
    loop {
        if let Some(c) = stream.next() {
            use self::ParseResult::*;
            match parse_char(c, stream) {
                Some(ast) => commands.push(ast),
                Ignore => continue,
                LoopEnd => break,
                x => return x,
            }
        } else {
            let err = ParseError::new("Missing ']' character");
            return ParseResult::Err(err);
        }
    }
    ParseResult::Some(Loop(commands))
}

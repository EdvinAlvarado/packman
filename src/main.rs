mod program;
use program::Program;
use std::io::ErrorKind;


fn main() -> Result<(), ErrorKind>{
    let mut program = Program::new();
    for (i, ar) in std::env::args().enumerate() {
        match i {
            0 => {},
            1 => {
                match &ar.as_str()[..2] {
                    "-S"    => {program.function = 'S'; program.parameters = ar.as_str()[2..].to_string();}
                    _       => {return Err(ErrorKind::InvalidInput);}
                }
            },
            _ => {program.packages.push(ar);}
        }
    }
    println!("{:?}", program);
    program.run()
}

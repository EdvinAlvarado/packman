mod program;
use program::Program;


fn main() -> Result<(), i32>{
    let mut program = Program::new();
    for (i, ar) in std::env::args().enumerate() {
        match i {
            0 => {},
            1 => {
                match &ar.as_str()[..2] {
                    "-S"    => {program.function = 'S'; program.parameters = ar.as_str()[2..].to_string();}
                    _       => {return Err(1);}
                }
            },
            _ => {program.packages.push(ar);}
        }
    }
    println!("{:?}", program);
    program.run()
}

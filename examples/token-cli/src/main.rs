use bitcoinsuite_core::{ser::BitcoinSer, tx::*};
use bytes::Bytes;
use clap::{command, Arg, ArgAction};

fn main() {
    let matches = command!() // requires `cargo` feature
        .arg(Arg::new("txtokens").action(ArgAction::Append))
        .get_matches();

    let args = matches
        .get_many::<String>("txtokens")
        .unwrap_or_default()
        .map(|v| v.as_str())
        .collect::<Vec<_>>();
    let tx_hex = hex::decode(&args[1]).unwrap();
    let bytes = &mut Bytes::from(tx_hex);
    let res = Transaction::deser(bytes);
    let tx = res.unwrap();

    tx.outputs.iter().for_each(|x| {
        if x.token.is_some() {
            print!("\n{:#?}\n", x.token.as_ref().unwrap());
        } else {
            print!("");
        }
    })
}

// use hex decoder  see hex string literal:

/*   println!("\nversion: {:#?}", tx.version);
tx.inputs.iter().for_each(|i| {
    println!("input");
    println!("  prev_out: {:#?}", i.prev_out);
    println!("  script sig: {:#?}", hex::encode(i.script.as_ref()));
    println!("  sequence: {:#?}", i.sequence);
});
tx.outputs.iter().for_each(|i| {
    println!("output");
    println!("  amount: {:#?}", i.value);
    println!("  script: {:#?}", hex::encode(i.script.as_ref()));
    println!("  token: {:#?}", i.token);
});
println!("  locktime: {:#?}", tx.locktime); */

use crate::*;

#[test]
fn args_view_round_trips_host_arguments() {
    let args = DenoArgs::new(vec![String::from("kali"), String::from("run")]);
    assert_eq!(
        args.as_slice(),
        &[String::from("kali"), String::from("run")]
    );
    assert_eq!(
        args.to_vec(),
        vec![String::from("kali"), String::from("run")]
    );
}

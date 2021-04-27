use anyhow::{anyhow, Context, Result};
use num_complex::Complex32;
use tap::Pipe;
use uhd::{self, Usrp};

/// Launch USRP in transmit mode
#[derive(argh::FromArgs)]
struct Args {
    /// dp the thing
    #[argh(switch, short = 't')]
    transmit: bool,

    /// dp the thing
    #[argh(switch, short = 'r')]
    receive: bool,
}

// Manual is here https://files.ettus.com/manual/page_converters.html
fn main() -> Result<()> {
    ofdm::logging::set_up_logging("ofdm");
    let cfg: Args = argh::from_env();

    match (cfg.transmit, cfg.receive) {
        (true, false) => start_sending(),
        (false, true) => start_receiving(),
        _ => panic!("Not a valid argument combination"),
    }
}

pub fn start_sending() -> Result<()> {
    let mut usrp = Usrp::find("")
        .context("Failed to open device list")?
        .drain(..)
        .next()
        .context("Failed to find a valid USRP to attach to")?
        .pipe(|addr| Usrp::open(&addr))
        .context("Failed to find properly open the USRP")?;

    // Set the stream type to be "fc32" which means "float complex 32"
    // See: https://files.ettus.com/manual/structuhd_1_1stream__args__t.html#a602a64b4937a85dba84e7f724387e252
    let stream = uhd::StreamArgs::<Complex32>::new("fc32");

    Ok(())
}

pub fn start_receiving() -> Result<()> {
    log::info!("Starting receive test");

    let mut usrp = Usrp::find("")
        .context("Failed to open device list")?
        .drain(..)
        .next()
        .context("Failed to find a valid USRP to attach to")?
        .pipe(|addr| Usrp::open(&addr))
        .context("Failed to find properly open the USRP")?;

    // Set the stream type to be "fc32" which means "float complex 32"
    // See: https://files.ettus.com/manual/structuhd_1_1stream__args__t.html#a602a64b4937a85dba84e7f724387e252

    let mut receiver = usrp.get_rx_stream(&uhd::StreamArgs::<Complex32>::new("fc32"))?;

    // let out_buffers = (0..receiver.num_channels())
    //     .map(|_| vec![0; 10000].into_boxed_slice())
    //     .collect::<Vec<_>>()
    //     .as_slice();

    let mut single_chan = vec![Complex32::default(); 1000].into_boxed_slice();
    receiver.receive_simple(single_chan.as_mut())?;

    log::info!("Samples received!");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send() -> anyhow::Result<()> {
        start_sending()
    }

    #[test]
    fn test_receive() -> anyhow::Result<()> {
        start_receiving()
    }
}

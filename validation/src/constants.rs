use bitcoin::{dogecoin::Network as DogecoinNetwork, Network as BitcoinNetwork};

use crate::BlockHeight;
use bitcoin::{CompactTarget, Target};
use ic_doge_types::BlockchainNetwork;

/// Expected number of blocks for 2 weeks (2_016) in Bitcoin.
pub const DIFFICULTY_ADJUSTMENT_INTERVAL_BITCOIN: BlockHeight = 6 * 24 * 14;

/// Expected number of blocks for 4 hours (240) in Dogecoin.
pub const DIFFICULTY_ADJUSTMENT_INTERVAL_DOGECOIN: BlockHeight = 4 * 60;

/// Needed to help test check for the 20 minute testnet/regtest rule
pub const TEN_MINUTES: u32 = 60 * 10;

/// Returns the maximum difficulty target depending on the network
pub fn max_target(network: &BlockchainNetwork) -> Target {
    use ic_doge_types::BlockchainNetwork::*;

    match network {
        Bitcoin(network) => match network {
            BitcoinNetwork::Bitcoin => Target::MAX_ATTAINABLE_MAINNET,
            BitcoinNetwork::Testnet | BitcoinNetwork::Testnet4 => Target::MAX_ATTAINABLE_TESTNET,
            BitcoinNetwork::Regtest => Target::MAX_ATTAINABLE_REGTEST,
            BitcoinNetwork::Signet => Target::MAX_ATTAINABLE_SIGNET,
            _ => unreachable!("Unsupported Bitcoin network variant: {:?}", network),
        },
        Dogecoin(network) => match network {
            DogecoinNetwork::Dogecoin => Target::MAX_ATTAINABLE_MAINNET_DOGE,
            DogecoinNetwork::Testnet => Target::MAX_ATTAINABLE_TESTNET_DOGE,
            DogecoinNetwork::Regtest => Target::MAX_ATTAINABLE_REGTEST_DOGE,
            _ => unreachable!("Unsupported Dogecoin network variant: {:?}", network),
        },
    }
}

/// Returns false iff PoW difficulty level of blocks can be
/// readjusted in the network after a fixed time interval.
pub fn no_pow_retargeting(network: &BlockchainNetwork) -> bool {
    use ic_doge_types::BlockchainNetwork::*;

    match network {
        Bitcoin(network) => match network {
            BitcoinNetwork::Bitcoin
            | BitcoinNetwork::Testnet
            | BitcoinNetwork::Testnet4
            | BitcoinNetwork::Signet => false,
            BitcoinNetwork::Regtest => true,
            _ => unreachable!("Unsupported Bitcoin network variant: {:?}", network),
        },
        Dogecoin(network) => match network {
            DogecoinNetwork::Dogecoin | DogecoinNetwork::Testnet => false,
            DogecoinNetwork::Regtest => true,
            _ => unreachable!("Unsupported Dogecoin network variant: {:?}", network),
        },
    }
}

/// Returns the PoW limit bits depending on the network
pub fn pow_limit_bits(network: &BlockchainNetwork) -> CompactTarget {
    use ic_doge_types::BlockchainNetwork::*;

    let bits = match network {
        Bitcoin(net) => match net {
            BitcoinNetwork::Bitcoin => 0x1d00ffff,
            BitcoinNetwork::Testnet | BitcoinNetwork::Testnet4 => 0x1d00ffff,
            BitcoinNetwork::Regtest => 0x207fffff,
            _ => unreachable!("Unsupported Bitcoin network variant: {:?}", net),
        },
        Dogecoin(net) => match net {
            DogecoinNetwork::Dogecoin => 0x1e0ffff0,
            DogecoinNetwork::Testnet => 0x1e0ffff0,
            DogecoinNetwork::Regtest => 0x207fffff,
            _ => unreachable!("Unsupported Dogecoin network variant: {:?}", net),
        },
    };
    CompactTarget::from_consensus(bits)
}

#[cfg(test)]
pub mod test {
    /// Mainnet 000000000000000000063108ecc1f03f7fd1481eb20f97307d532a612bc97f04
    pub const MAINNET_HEADER_586656: &str ="00008020cff0e07ab39db0f31d4ded81ba2339173155b9c57839110000000000000000007a2d75dce5981ec421a54df706d3d407f66dc9170f1e0d6e48ed1e8a1cad7724e9ed365d083a1f17bc43b10a";
    /// Mainnet 0000000000000000000d37dfef7fe1c7bd22c893dbe4a94272c8cf556e40be99
    pub const MAINNET_HEADER_705600: &str = "0400a0205849eed80b320273a73d39933c0360e127d15036a69d020000000000000000006cc2504814505bb6863d960599c1d1f76a4768090ac15b0ad5172f5a5cd918a155d86d6108040e175daab79e";
    /// Mainnet 0000000000000000000567617f2101a979d04cff2572a081aa5f29e30800ab75
    pub const MAINNET_HEADER_705601: &str = "04e0002099be406e55cfc87242a9e4db93c822bdc7e17fefdf370d000000000000000000eba036bca22654014f363f3019d0f08b3cdf6b2747ab57eff2e6dc1da266bc0392d96d6108040e176c6624cd";
    /// Mainnet 00000000000000000001eea12c0de75000c2546da22f7bf42d805c1d2769b6ef
    pub const MAINNET_HEADER_705602: &str = "0400202075ab0008e3295faa81a07225ff4cd079a901217f616705000000000000000000c027a2615b11b4c75afc9e42d1db135d7124338c1f556f6a14d1257a3bd103a5f4dd6d6108040e1745d26934";

    /// Testnet 00000000000000e23bb091a0046e6c73160db0a71aa052c20b10ff7de7554f97
    pub const TESTNET_HEADER_2132555: &str = "004000200e1ff99438666c67c649def743fb82117537c2017bcc6ad617000000000000007fa40cf82bf224909e3174281a57af2eb3a4a2a961d33f50ec0772c1221c9e61ddfdc061ffff001a64526636";
    /// Testnet 00000000383cd7fff4692410ccd9bd6201790043bb41b93bacb21e9b85620767
    pub const TESTNET_HEADER_2132556: &str = "00000020974f55e77dff100bc252a01aa7b00d16736c6e04a091b03be200000000000000c44f2d69fc200c4a2211885000b6b67512f42c1bec550f3754e103b6c4046e05a202c161ffff001d09ec1bc4";

    /// Mainnet 0c120ab190655673a709bc92ad86f80dc1cd9f11f9e0f09ebc5e6a3058b73002
    pub const MAINNET_HEADER_DOGE_17: &str = "01000000fbc172c83b7e535390cfd7807118a7fc799cdbda9da0cbd390f4b70c0f62c2fb155fa2e0ad11cfd91cd0f47049c0fcf5dfabd2fe1a3a406c0350e89f14618bb1f4eda352f0ff0f1e00067505";
    /// Mainnet da0e2362cc1d1cd48c8eb70e578c97f00d9a530985ba36027eb7e3fba98c74ae
    pub const MAINNET_HEADER_DOGE_18: &str = "010000000230b758306a5ebc9ef0e0f9119fcdc10df886ad92bc09a773566590b10a120ca96ac7b3a8ef18a68f1044aef152724403bb6bb6e2e44bdb26395a6f00ec858df6eda352f0ff0f1e0002c935";
    /// Mainnet 7b8f3d7006952b2d6663967ed2b20226db8c1c3c392c0fe1da14cca43b55e344
    pub const MAINNET_HEADER_DOGE_23505: &str ="01000000f6ee239bf5264e3a178517140db01eda71343bd7defdccee54850a561dbd474bc1e1706d919aaba102e68d3fa67d0eb58846c6770ce30c6c17d8410c00b30628f355b752a6c2001c0058caa9";
    /// Mainnet f0b769c6eb33f9671e0f520df1bacb76d91fa2d41d6e19d3b6becd9a6daa45c1
    pub const MAINNET_HEADER_DOGE_151555: &str = "02000000376eb99d96fb9680c3dda3d79f0595996ced278edeb805166d86fdf69d762aa3802bc1f0424689bdc1b12b16509353bf94e32efcc20c68bbdcd1b1e63dd2217a38e72d531470271b00514e69";
    /// Mainnet 3b595392744d34544c300a886577f0bd839aeb788e3e8e19138e6092eb5c2ad6
    pub const MAINNET_HEADER_DOGE_151556: &str = "02000000c145aa6d9acdbeb6d3196e1dd4a21fd976cbbaf10d520f1e67f933ebc669b7f0748f9b1f40c1a60740c8f47d506c6de6b9f415d00c8a9c4484ef36abe593a39fe4e72d5301cf241b00220241";
    /// Mainnet d3b4205b9cab0c969d0e96ff924ab4e3acd8779c2ce1669b94c98d6f2f0365f4
    pub const MAINNET_HEADER_DOGE_151557: &str = "02000000d62a5ceb92608e13198e3e8e78eb9a83bdf07765880a304c54344d749253593b9debfe3e9ead5f19238f3ec5bb321de36acf69f0680d0231c4612961fd2e0fa91ae82d53b4652d1b00adb73b";

    /// Testnet c569d72a51580678c8f111456eb259b5fc1c35c20679b9effc25e9d15885ebe8
    pub const TESTNET_HEADER_DOGE_85313: &str = "02000000df682c3a57866f7e9ba1ed021495af6132ae650b73b33e6a6539b68271a1f97112e15b65d5288659a2096d166a8b0cd706c896c40d27fb975f20cae7ed3c397af3097b53ffff0f1e92ed0000";
    /// Testnet 1eff6e8cfcbb6cffeb80321db8698fb3397027d13f9d71c56c12fe41343cef89
    pub const TESTNET_HEADER_DOGE_710878: &str = "03006200d9abc3f6038a7ce1c155f8f5ed55e017830e597d3394619e2101f848e877961be44401cf4217b95e03c472561c1b1f4daf41f9f9aac0b7f8535573f4c9e5c3df1701e855ffff0f1e1eed0c00";
}

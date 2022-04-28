import * as anchor from '@project-serum/anchor';
import { PublicKey } from '@solana/web3.js';

export interface GlobalPool {
    admin: PublicKey,
    totalAmount: anchor.BN,
    adventureRate: anchor.BN,
    scientistRate: anchor.BN,
    doctorRate: anchor.BN,
    specialistRate: anchor.BN,
    commanderRate: anchor.BN,
    normalRate: anchor.BN
}

export interface StakedNFT {
    nftAddr: PublicKey,
    stakeTime: anchor.BN,
    rewardTime: anchor.BN,
    lockTime: anchor.BN,
    rate: anchor.BN,
    model: anchor.BN
}

export interface UserPool {
    owner: PublicKey,
    itemCount: anchor.BN,
    items: StakedNFT[],
    rewardTime: anchor.BN,
    pendingReward: anchor.BN,
}

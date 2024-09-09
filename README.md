# NFT-staking-Death
Multiple Factor Rewards Staking program for Astronaut NFT collections. This has 3 models to stake NFTs.

<h4> 📞 Cᴏɴᴛᴀᴄᴛ ᴍᴇ Oɴ ʜᴇʀᴇ: 👆🏻 </h4>

<p> 
    <a href="mailto:nakao95911@gmail.com" target="_blank">
        <img alt="Email"
        src="https://img.shields.io/badge/Email-00599c?style=for-the-badge&logo=gmail&logoColor=white"/>
    </a>
     <a href="https://x.com/solkeen" target="_blank"><img alt="Twitter"
        src="https://img.shields.io/badge/Twitter-000000?style=for-the-badge&logo=x&logoColor=white"/></a>
    <a href="https://discordapp.com/users/415742962119606272" target="_blank"><img alt="Discord"
        src="https://img.shields.io/badge/Discord-7289DA?style=for-the-badge&logo=discord&logoColor=white"/></a>
    <a href="https://t.me/soIkeen" target="_blank"><img alt="Telegram"
        src="https://img.shields.io/badge/Telegram-26A5E4?style=for-the-badge&logo=telegram&logoColor=white"/></a>
</p>

## Install Dependencies
- Install `node` and `yarn`
- Install `ts-node` as global command
- Confirm the solana wallet preparation: `/home/fury/.config/solana/id.json` in test case

## Usage
- Main script source for all functionality is here: `/cli/script.ts`
- Program account types are declared here: `/cli/types.ts`
- Idl to make the JS binding easy is here: `/cli/staking_program.json`

Able to test the script functions working in this way.
- Change commands properly in the main functions of the `script.ts` file to call the other functions
- Confirm the `ANCHOR_WALLET` environment variable of the `ts-node` script in `package.json`
- Run `yarn ts-node`

## Features

### As a Smart Contract Owner
For the first time use, the Smart Contract Owner should `initialize` the Smart Contract for global account allocation.
- `initProject`
 
Recall `initialize` function for update the Threshold values after change the constants properly
- `initProject` 

Maintain the Reward token($COSMIC) vault's balance
- `REWARD_TOKEN_MINT` is the reward token mint (for test).
- `rewardVault` is the reward token account for owner. The owner should have the token's `Mint Authority` or should `Fund` regularly.

This is current test value. Should be revised properly.
- `EPOCH` = 86400                                   // A day 
- According to the rank of NFTs, there reward amount will be changed automatically following the below logic.
```
Model 1: 
Trait based 
Adventurer: 20 $coin a day 
Scientist: 25 $coin a day 
Doctor: 30 coin a day 
Mission Specialist: 35 $coin a day 
Commander: 40 $coin a day
If you claim before 15 days, you will receive 75% and after 15 days, will receive 100%.

Model 2 
Rewards are paid daily and they are the same for all NFTS, so x $coin a day for everything 

Model 3 
7 days x $coin 
14 days 1.5x $coin 
30 days+ 2x $coin
```

### As a NFT Holder
Stake Shred Collection NFTs with NFT `mint address` and a boolean parameter weather the NFT is Legendary NFT.
- `stakeNft`

### As a Staker
Unstake their staked NFTs with `mint address` and get rewards. ( Calculate generated reward by this NFT too )
- `withdrawNft`

Claim reward to receive generated $COSMIC from their staking.
- `claimReward`

Claim reward to receive generated $COSMIC from their staking for all NFTs.
- `claimRewardAll`

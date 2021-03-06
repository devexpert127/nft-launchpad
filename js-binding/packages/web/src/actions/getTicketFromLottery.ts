import { Connection, Keypair, TransactionInstruction } from '@solana/web3.js';
import {
  WalletSigner,
  sendTransactionWithRetry,
  getTicket,
  LotteryData,
  createTokenAccountIfNotExist,
  WRAPPED_SOL_MINT,
  programIds,
  toPublicKey,
} from '@oyster/common';
import { WalletNotConnectedError } from '@solana/wallet-adapter-base';
import { AccountLayout, Token } from '@solana/spl-token';

// This command makes an Lottery
export async function getTicketFromLottery(
  connection: Connection,
  wallet: WalletSigner,
  lottery: string,
  lotteryData: LotteryData,
): Promise<{
  txid: string;
  slot: number;
}> {
  if (!wallet.publicKey) throw new WalletNotConnectedError();

  const accountRentExempt = await connection.getMinimumBalanceForRentExemption(
    AccountLayout.span,
  );

  const instructions: TransactionInstruction[] = [];

  const signers: Keypair[] = [];
  const ticket = new Keypair();

  const seed = toPublicKey(lottery).toBytes();
  const walletByte = wallet.publicKey.toBytes();

  for (let i = 0; i < 32; i++) {
    seed[i] += walletByte[i];
  }

  const ticketPocket = Keypair.fromSeed(seed);

  signers.push(ticket);

  const userWsolToken = await createTokenAccountIfNotExist(
    connection,
    null,
    wallet.publicKey,
    WRAPPED_SOL_MINT.toBase58(),
    lotteryData.ticketPrice.toNumber() + accountRentExempt,
    instructions,
    signers,
  );

  signers.push(ticketPocket);
  getTicket(
    ticket.publicKey.toBase58(),
    wallet.publicKey.toBase58(),
    ticketPocket.publicKey.toBase58(),
    userWsolToken.toBase58(),
    lotteryData.tokenPool,
    lotteryData.tokenMint,
    wallet.publicKey.toBase58(),
    lottery,
    instructions,
  );

  instructions.push(
    Token.createCloseAccountInstruction(
      programIds().token,
      userWsolToken,
      wallet.publicKey,
      wallet.publicKey,
      signers,
    ),
  );

  const { txid, slot } = await sendTransactionWithRetry(
    connection,
    wallet,
    instructions,
    signers,
    'single',
  );

  return { txid, slot };
}

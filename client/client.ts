import {
  Keypair,
  Connection,
  PublicKey,
  Transaction,
  TransactionInstruction,
  sendAndConfirmTransaction,
} from "@solana/web3.js";

// Connection to the Solana cluster
const connection = new Connection("https://api.devnet.solana.com", "confirmed");
const progId = new PublicKey("Fay317m7Geo3eTzxFJd8EFK1tab8sciSKwDhEib2cArg");

const wallet = Keypair.fromSecretKey(
  Uint8Array.from([
    53, 201, 129, 78, 216, 92, 188, 182, 221, 56, 8, 158, 2, 64, 252, 100, 223,
    173, 174, 29, 83, 20, 152, 245, 255, 2, 224, 224, 74, 208, 73, 204, 166,
    183, 98, 86, 248, 114, 83, 73, 201, 202, 208, 139, 64, 9, 130, 146, 205,
    147, 72, 225, 244, 174, 102, 78, 78, 60, 20, 210, 242, 13, 136, 124,
  ])
);

// The public key of the GPU registry account
const gpuRegistryPubkey = new PublicKey(
  "CDns6TVDnPXsZTxGmMz2PjVK6kM156yuutReTGBsp3to"
);

import { serialize, deserialize, deserializeUnchecked } from "borsh";
import { Buffer } from "buffer";

class Assignable {
  constructor(properties) {
    Object.keys(properties).map((key) => {
      return (this[key] = properties[key]);
    });
  }
}
class Payload extends Assignable {}
const payloadSchema = new Map([
  [
    Payload,
    {
      kind: "struct",
      fields: [["key", "string"]],
    },
  ],
]);
enum InstructionVariant {
  InitializeAccount = 0,
  MintKeypair,
  TransferKeypair,
  BurnKeypair,
}

async function mintKV(mintKey: string): Promise<string> {
  // Construct the payload
  const mint = new Payload({
    key: mintKey, // 'ts key'
  });
  console.log(mint);

  // Serialize the payload
  const mintSerBuf = Buffer.from(serialize(payloadSchema, mint));

  const instruction = new TransactionInstruction({
    data: mintSerBuf,
    keys: [
      { pubkey: gpuRegistryPubkey, isSigner: false, isWritable: true },
      { pubkey: wallet.publicKey, isSigner: false, isWritable: false },
    ],
    programId: progId,
  });

  // Send Solana Transaction
  const transactionSignature = await sendAndConfirmTransaction(
    connection,
    new Transaction().add(instruction),
    [wallet],
    {
      commitment: "singleGossip",
      preflightCommitment: "singleGossip",
    }
  );
  console.log("Signature = ", transactionSignature);
  return transactionSignature;
}

const gpuModel = "abc3008";
mintKV(gpuModel)
  .then(() => console.log("Registration complete"))
  .catch((error) => console.error("Error registering GPU:", error));

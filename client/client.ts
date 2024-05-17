import {
  Keypair,
  Connection,
  PublicKey,
  Transaction,
  TransactionInstruction,
  sendAndConfirmTransaction,
  Account,
} from "@solana/web3.js";
import { serialize } from "borsh";
import { Buffer } from "buffer";

const connection = new Connection("https://api.devnet.solana.com", "confirmed");
const progId = new PublicKey("Fay317m7Geo3eTzxFJd8EFK1tab8sciSKwDhEib2cArg");

const wallet = Keypair.fromSecretKey(
  Uint8Array.from([
    45, 134, 186, 18, 67, 205, 31, 12, 89, 188, 95, 14, 215, 2, 218, 188, 95,
    222, 195, 124, 224, 246, 220, 58, 188, 174, 159, 80, 187, 148, 113, 26, 9,
    36, 120, 57, 190, 48, 158, 152, 109, 243, 222, 194, 142, 254, 82, 208, 79,
    58, 166, 180, 228, 192, 147, 120, 26, 113, 187, 131, 235, 214, 8, 21,
  ])
);

const gpuRegistryPubkey = new PublicKey(
  "cgvYffFMGJbj8Rvicsw6dDHT1dhMpSvE4DNvkSx9WLc"
);

class Payload {
  variant: number;
  model: string;
  constructor(variant: number, model: string) {
    this.variant = variant;
    this.model = model;
  }
}

const payloadSchema = new Map([
  [
    Payload,
    {
      kind: "struct",
      fields: [
        ["variant", "u8"],
        ["model", "string"],
      ],
    },
  ],
]);

async function initialize_account(model: string): Promise<string> {
  const payload = new Payload(0, model);
  const serializedPayload = Buffer.from(serialize(payloadSchema, payload));
  const accountPublicKey = new PublicKey(
    "Dea7vK8jhKqNSV6EaHePiJ5GW4tUQbgutSrGBFuVTxwm"
  );
  const data = Buffer.from([0]);
  const instruction = new TransactionInstruction({
    data: serializedPayload,
    keys: [{ pubkey: accountPublicKey, isSigner: false, isWritable: true }],
    programId: progId,
  });

  const transactionSignature = await sendAndConfirmTransaction(
    connection,
    new Transaction().add(instruction),
    [wallet],
    {
      commitment: "confirmed",
      preflightCommitment: "confirmed",
    }
  );
  console.log("Transaction Signature =", transactionSignature);
  return transactionSignature;
}

async function registerGPU(model: string): Promise<string> {
  //  payload for RegisterGpu
  const payload = new Payload(1, model); // Variant 1 corresponds to RegisterGpu
  console.log("Payload:", payload);

  // payload
  const serializedPayload = Buffer.from(serialize(payloadSchema, payload));

  const instruction = new TransactionInstruction({
    data: serializedPayload,
    keys: [
      { pubkey: gpuRegistryPubkey, isSigner: false, isWritable: true },
      { pubkey: wallet.publicKey, isSigner: true, isWritable: false },
    ],
    programId: progId,
  });

  // Send Solana Transaction
  const transactionSignature = await sendAndConfirmTransaction(
    connection,
    new Transaction().add(instruction),
    [wallet],
    {
      commitment: "confirmed",
      preflightCommitment: "confirmed",
    }
  );
  console.log("Transaction Signature =", transactionSignature);
  return transactionSignature;
}

const gpuModel = "Rtx3030";
initialize_account(gpuModel);
registerGPU(gpuModel)
  .then(() => console.log("GPU registration complete"))
  .catch((error) => console.error("Error registering GPU:", error));

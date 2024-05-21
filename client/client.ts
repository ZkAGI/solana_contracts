import {
  Keypair,
  Connection,
  PublicKey,
  Transaction,
  TransactionInstruction,
  sendAndConfirmTransaction,
  SystemProgram,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import { serialize } from "borsh";
import { Buffer } from "buffer";

const connection = new Connection("http://127.0.0.1:8899", "confirmed");
const progId = new PublicKey("7MyWqb1JDgHGfkbuyGFvFaPQwLDKJyX2hymhj9NMSYuh");
const authorityId = new PublicKey(
  "G8XnWUNznBctaJ6iGz39S5ZkZCjeaSHCEtyvpJEaBokQ"
);
const wallet = Keypair.fromSecretKey(
  Uint8Array.from([
    20, 242, 196, 8, 235, 129, 57, 129, 208, 151, 213, 166, 24, 31, 163, 9, 122,
    142, 48, 254, 92, 61, 18, 240, 244, 36, 192, 244, 219, 157, 131, 76, 224,
    205, 45, 135, 32, 39, 151, 130, 126, 140, 107, 149, 38, 34, 16, 109, 7, 125,
    88, 26, 122, 237, 107, 184, 238, 0, 23, 59, 222, 24, 52, 205,
  ])
);

const ownerId = new PublicKey("CDns6TVDnPXsZTxGmMz2PjVK6kM156yuutReTGBsp3to");

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
const [storagePDA, _] = await PublicKey.findProgramAddressSync(
  [Buffer.from("storagePool"), authorityId.toBuffer()],
  progId
);

async function initialize_account(model: string): Promise<string> {
  const payload = new Payload(0, model);
  const serializedPayload = Buffer.from(serialize(payloadSchema, payload));
  const storageAccount = Keypair.generate();
  const instruction = new TransactionInstruction({
    data: serializedPayload,
    keys: [
      { pubkey: wallet.publicKey, isSigner: true, isWritable: false },
      { pubkey: storagePDA, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: true },
    ],
    programId: progId,
  });

  try {
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
  } catch (error) {
    console.error("Error registering GPU:", error);
    throw error;
  }
}

async function create_entry(model: string): Promise<string> {
  const payload = new Payload(1, model);
  const serializedPayload = Buffer.from(serialize(payloadSchema, payload));
  const [derivedPDA, _] = await PublicKey.findProgramAddressSync(
    [Buffer.from("Entry"), Buffer.from(model), authorityId.toBuffer()],
    progId
  );
  const storageAccount = Keypair.generate();
  // const serializedPayload = Buffer.from(serialize(payloadSchema, payload));
  const instruction = new TransactionInstruction({
    data: serializedPayload,
    keys: [
      { pubkey: wallet.publicKey, isSigner: true, isWritable: false },
      { pubkey: storagePDA, isSigner: false, isWritable: true },
      { pubkey: derivedPDA, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: true },
    ],
    programId: progId,
  });

  try {
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
  } catch (error) {
    console.error("Error registering GPU:", error);
    throw error; // Rethrow the error for proper handling in the calling function.
  }
}

const gpuModel = "yuhh";
create_entry(gpuModel)
  // registerGPU(gpuModel)
  .then(() => console.log("GPU registration complete"))
  .catch((error) => console.error("Error registering GPU:", error));

// No imports needed: web3, borsh, pg and more are globally available

/**
 * The state of a GPU registry account managed by the program
 */
class GpuRegistryAccount {
  owner;
  model;

  constructor(fields = { owner: "", model: "" }) {
    this.owner = fields.owner;
    this.model = fields.model;
  }
}

/**
 * Borsh schema definition for GPU registry accounts
 */
const GpuRegistrySchema = new Map([
  [
    GpuRegistryAccount,
    {
      kind: "struct",
      fields: [
        ["owner", "string"],
        ["model", "string"],
      ],
    },
  ],
]);

/**
 * The expected size of each GPU registry account.
 */
const GPU_REGISTRY_SIZE = borsh.serialize(
  GpuRegistrySchema,
  new GpuRegistryAccount()
).length;

describe("GPU Registration", () => {
  it("should create a new GPU registry account", async () => {
    // Create a new GPU registry account
    const gpuRegistryAccountKp = new web3.Keypair();
    const lamports = await pg.connection.getMinimumBalanceForRentExemption(
      GPU_REGISTRY_SIZE
    );
    const createGpuRegistryAccountIx = web3.SystemProgram.createAccount({
      fromPubkey: pg.wallet.publicKey,
      lamports,
      newAccountPubkey: gpuRegistryAccountKp.publicKey,
      programId: pg.PROGRAM_ID,
      space: GPU_REGISTRY_SIZE,
    });

    // Create transaction and add the create account instruction
    const tx = new web3.Transaction();
    tx.add(createGpuRegistryAccountIx);

    // Send and confirm the transaction
    await web3.sendAndConfirmTransaction(pg.connection, tx, [
      pg.wallet.keypair,
      gpuRegistryAccountKp,
    ]);

    // Fetch the GPU registry account
    const gpuRegistryAccount = await pg.connection.getAccountInfo(
      gpuRegistryAccountKp.publicKey
    );

    // Assertions
    assert.strictEqual(gpuRegistryAccount.lamports, lamports);
    assert(gpuRegistryAccount.owner.equals(pg.PROGRAM_ID));
    assert(gpuRegistryAccount.data.length === GPU_REGISTRY_SIZE); // Account data should be empty
  });

  it("should register a new GPU", async () => {
    // Create a new GPU registry account
    const gpuRegistryAccountKp = new web3.Keypair();
    const lamports = await pg.connection.getMinimumBalanceForRentExemption(
      GPU_REGISTRY_SIZE
    );
    const createGpuRegistryAccountIx = web3.SystemProgram.createAccount({
      fromPubkey: pg.wallet.publicKey,
      lamports,
      newAccountPubkey: gpuRegistryAccountKp.publicKey,
      programId: pg.PROGRAM_ID,
      space: GPU_REGISTRY_SIZE,
    });

    // Register a new GPU
    const gpuModel = "GTX 3080";
    const ownerPublicKey = new web3.PublicKey(
      "CDns6TVDnPXsZTxGmMz2PjVK6kM156yuutReTGBsp3to"
    ); // Replace with actual owner's public key
    const registerGpuIx = new web3.TransactionInstruction({
      keys: [
        {
          pubkey: gpuRegistryAccountKp.publicKey,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: ownerPublicKey,
          isSigner: false,
          isWritable: false,
        },
      ],
      programId: pg.PROGRAM_ID,
      data: Buffer.from(`RegisterGpu,${ownerPublicKey},${gpuModel}`),
    });

    // Create transaction and add instructions
    const tx = new web3.Transaction();
    tx.add(createGpuRegistryAccountIx, registerGpuIx);

    // Send and confirm the transaction
    await web3.sendAndConfirmTransaction(pg.connection, tx, [
      pg.wallet.keypair,
      gpuRegistryAccountKp,
    ]);
    console.log("done");
    // Fetch the GPU registry account
    const gpuRegistryAccount = await pg.connection.getAccountInfo(
      gpuRegistryAccountKp.publicKey
    );

    // Deserialize the account data
    const deserializedAccountData = borsh.deserialize(
      GpuRegistrySchema,
      GpuRegistryAccount,
      gpuRegistryAccount.data
    );

    // Assertions
    assert(deserializedAccountData.model === gpuModel); // Model name should match the registered GPU
  });
});

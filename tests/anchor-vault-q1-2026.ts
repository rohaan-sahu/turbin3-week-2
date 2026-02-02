import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AnchorVaultQ12026 } from "../target/types/anchor_vault_q1_2026";
import assert from "node:assert";
import { expect } from "chai";

describe("anchor-vault-q1-2026", () => {
  // Configure the client to use the local cluster.
  //anchor.setProvider(anchor.AnchorProvider.env());

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.anchorVaultQ12026 as Program<AnchorVaultQ12026>;
  // provider
  const user = anchor.AnchorProvider.env().wallet.publicKey;


  // Derived PDA
  const [vaultStatePda,stateBump] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("state"),user.toBuffer()],
    program.programId
  );

  const [vaultPda,vaultBump] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("vault"),vaultStatePda.toBuffer()],
    program.programId
  );

  console.log("stateBump: ",stateBump );
  console.log("vaultBump: ",vaultBump );

  // Pre-test setup
  before(async () => {
    // Airdrop  for fees
    await anchor.AnchorProvider.env().connection.requestAirdrop(
      user,
      10*anchor.web3.LAMPORTS_PER_SOL
    );

    // Confirm airdrop
    await new Promise(resolve => setTimeout(resolve,1000));
  });

  it("Initialize the Vault.", async () => {
    await program.methods
      .initialize()
      .accountsStrict({
        systemProgram: anchor.web3.SystemProgram.programId,
        user: user,
        vaultState: vaultStatePda,
        vault: vaultPda,
      })
      .rpc();
    
    const vaultState = await program.account.vaultState.fetch(vaultStatePda);

    // expect(vaultState.vaultBump).to.equal(vaultBump);
    // expect(vaultState.stateBump).to.equal(stateBump);

    assert.strictEqual(vaultState.vaultBump,vaultBump,"1");
    assert.strictEqual(vaultState.stateBump,stateBump,"2");

    const vaultBalance = await anchor.AnchorProvider.env().connection.getBalance(vaultPda);
    const rentExempt = await anchor.AnchorProvider.env().connection.getMinimumBalanceForRentExemption(0);

    assert.strictEqual(vaultBalance,rentExempt,"3");

  });

  it("Deposit SOL into the vault", async () => {
    const depositAmount = 1 * anchor.web3.LAMPORTS_PER_SOL; // 1 SOL

    const initialVaultBalance = await provider.connection.getBalance(vaultPda);
    const initialUserBalance = await provider.connection.getBalance(user);

    await program.methods
      .deposit(new anchor.BN(depositAmount))
      .accountsStrict({
        user: user,
        vault: vaultPda,
        vaultState: vaultStatePda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const finalVaultBalance = await provider.connection.getBalance(vaultPda);
    const finalUserBalance = await provider.connection.getBalance(user);

    assert.strictEqual(finalVaultBalance,(initialVaultBalance + depositAmount),"4");
    // User balance decreases by amount - fees
    assert.strictEqual(finalUserBalance,(initialUserBalance - depositAmount - 5000),"5");
  });

  it("Withdraw SOL from the vault", async () => {
    const withdrawAmount = 0.5 * anchor.web3.LAMPORTS_PER_SOL; // 0.5 SOL

    const initialVaultBalance = await provider.connection.getBalance(vaultPda);
    const initialUserBalance = await provider.connection.getBalance(user);

    await program.methods
      .withdraw(new anchor.BN(withdrawAmount))
      .accountsStrict({
        user: user,
        vault: vaultPda,
        vaultState: vaultStatePda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const finalVaultBalance = await provider.connection.getBalance(vaultPda);
    const finalUserBalance = await provider.connection.getBalance(user);

    assert.strictEqual(finalVaultBalance,(initialVaultBalance - withdrawAmount));
    // User balance increases by amount - fees
    assert.strictEqual(finalUserBalance,(initialUserBalance + withdrawAmount - 5000));
  });

  it("Close the vault", async () => {
    const initialVaultBalance = await provider.connection.getBalance(vaultPda);
    const initialVaultStateBalance = await provider.connection.getBalance(vaultStatePda);
    const initialUserBalance = await provider.connection.getBalance(user);

    await program.methods
      .close()
      .accountsStrict({
        user: user,
        vault: vaultPda,
        vaultState: vaultStatePda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const finalUserBalance = await provider.connection.getBalance(user);

    // Vault should be 0
    const vaultBal = await provider.connection.getBalance(vaultPda);
    assert.strictEqual(vaultBal,0);

    // VaultState should be closed (null)
    const vaultStateInfo = await provider.connection.getAccountInfo(vaultStatePda);
    assert.strictEqual(vaultStateInfo,null);

    // User gets back the remaining balance - fees
    assert.strictEqual(finalUserBalance,(initialUserBalance + initialVaultBalance + initialVaultStateBalance - 5000));

    
  });
});

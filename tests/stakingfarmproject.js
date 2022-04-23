const anchor = require('@project-serum/anchor')

const main = async () => {
  console.log("Stating test...")
  //this is esentially the same as brownie network show-active which will grab either the local, dev/test, main net.. 
  anchor.setProvider(anchor.Provider.env());
  const program = anchor.workspace.Stakingfarmproject;
  const tx = await program.rpc.initialize();

  console.log("Your transaction signature", tx);
}

const runMain = async () => {
  try {
    await main();
    process.exit(0);
  } catch (error) {
    console.error(error);
    process.exit(1);
  }
};

runMain();
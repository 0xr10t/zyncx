import anchor from "@coral-xyz/anchor";

module.exports = async function (provider: any) {
  anchor.setProvider(provider);

  // Add your deploy script here.
};

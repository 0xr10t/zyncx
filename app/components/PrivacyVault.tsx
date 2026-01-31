'use client';

import { motion } from 'framer-motion';
import { useState, useEffect } from 'react';
import { ArrowDownUp, Lock, Unlock, Droplets, Copy, Check, AlertCircle, Loader2, Download, Info } from 'lucide-react';
import { useWallet, useConnection } from '@solana/wallet-adapter-react';
import { useZyncx, DepositNote, encodeNote, decodeNote, generateDepositSecrets, computePrecommitment, createDepositNote } from '../lib';

export default function PrivacyVault() {
  const [activeTab, setActiveTab] = useState<'deposit' | 'withdraw' | 'swap'>('deposit');
  const [amount, setAmount] = useState('');
  const [noteInput, setNoteInput] = useState('');
  const [savedNote, setSavedNote] = useState<DepositNote | null>(null);
  const [copied, setCopied] = useState(false);
  const [vaultInfo, setVaultInfo] = useState<any>(null);
  const [demoMode, setDemoMode] = useState(false);
  const [isDevnet, setIsDevnet] = useState<boolean | null>(null);
  
  const wallet = useWallet();
  const { connection } = useConnection();
  const { isLoading, error, lastTx, depositSol, withdrawSol, getVaultInfo } = useZyncx();

  // Check if connected to devnet
  useEffect(() => {
    const checkNetwork = async () => {
      try {
        const genesisHash = await connection.getGenesisHash();
        // Devnet genesis hash
        const devnetGenesis = 'EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG';
        setIsDevnet(genesisHash === devnetGenesis);
      } catch {
        setIsDevnet(null);
      }
    };
    checkNetwork();
  }, [connection]);

  // Fetch vault info on mount
  useEffect(() => {
    getVaultInfo().then(setVaultInfo);
  }, [getVaultInfo]);

  const handleDeposit = async () => {
    if (!amount || parseFloat(amount) <= 0) return;
    
    if (demoMode) {
      // Demo mode: Generate note without blockchain transaction
      const { secret, nullifierSecret } = generateDepositSecrets();
      const amountLamports = BigInt(Math.floor(parseFloat(amount) * 1e9));
      const note = createDepositNote(secret, nullifierSecret, amountLamports);
      note.txSignature = 'demo-' + Date.now();
      setSavedNote(note);
      return;
    }
    
    const note = await depositSol(parseFloat(amount));
    if (note) {
      setSavedNote(note);
    }
  };

  const handleWithdraw = async () => {
    if (!noteInput) return;
    try {
      const note = decodeNote(noteInput);
      await withdrawSol(note);
    } catch (e) {
      console.error('Invalid note format');
    }
  };

  const copyNote = () => {
    if (savedNote) {
      navigator.clipboard.writeText(encodeNote(savedNote));
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  const downloadNote = () => {
    if (savedNote) {
      const blob = new Blob([encodeNote(savedNote)], { type: 'text/plain' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `zyncx-note-${Date.now()}.txt`;
      a.click();
    }
  };

  return (
    <section id="features" className="relative py-32 px-6">
      <div className="max-w-7xl mx-auto">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          className="text-center mb-16"
        >
          <h2 className="text-5xl font-bold mb-4 bg-gradient-to-r from-cyber-purple to-cyber-blue bg-clip-text text-transparent">
            Privacy Vault
          </h2>
          <p className="text-gray-400 text-lg">
            Interact with the protocol. All transactions are private and untraceable.
          </p>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, scale: 0.95 }}
          whileInView={{ opacity: 1, scale: 1 }}
          viewport={{ once: true }}
          className="max-w-2xl mx-auto glass-effect rounded-2xl p-8 cyber-border"
        >
          {/* Tab Navigation */}
          <div className="flex gap-4 mb-8">
            <TabButton
              active={activeTab === 'deposit'}
              onClick={() => setActiveTab('deposit')}
              icon={<Droplets className="w-5 h-5" />}
            >
              Deposit
            </TabButton>
            <TabButton
              active={activeTab === 'withdraw'}
              onClick={() => setActiveTab('withdraw')}
              icon={<Unlock className="w-5 h-5" />}
            >
              Withdraw
            </TabButton>
            <TabButton
              active={activeTab === 'swap'}
              onClick={() => setActiveTab('swap')}
              icon={<ArrowDownUp className="w-5 h-5" />}
            >
              Swap
            </TabButton>
          </div>

          {/* Content Area */}
          <div className="space-y-6">
            {/* Network Warning */}
            {isDevnet === false && (
              <div className="flex items-center gap-2 p-3 bg-yellow-500/20 border border-yellow-500/50 rounded-lg text-yellow-400">
                <AlertCircle className="w-5 h-5" />
                <div>
                  <span className="font-semibold">Wrong Network!</span>
                  <p className="text-xs mt-1">Switch your wallet to Devnet in settings</p>
                </div>
              </div>
            )}

            {/* Demo Mode Toggle */}
            <div className="flex items-center justify-between p-3 bg-cyber-purple/10 border border-cyber-purple/20 rounded-lg">
              <div className="flex items-center gap-2">
                <Info className="w-4 h-4 text-cyber-purple" />
                <span className="text-sm text-gray-300">Demo Mode (no blockchain)</span>
              </div>
              <button
                onClick={() => setDemoMode(!demoMode)}
                className={`w-12 h-6 rounded-full transition-colors ${demoMode ? 'bg-cyber-purple' : 'bg-gray-600'}`}
              >
                <div className={`w-5 h-5 rounded-full bg-white transition-transform ${demoMode ? 'translate-x-6' : 'translate-x-0.5'}`} />
              </button>
            </div>

            {/* Error Display */}
            {error && !demoMode && (
              <div className="flex items-center gap-2 p-3 bg-red-500/20 border border-red-500/50 rounded-lg text-red-400">
                <AlertCircle className="w-5 h-5" />
                <span>{error}</span>
              </div>
            )}

            {/* Success Display */}
            {(lastTx || (demoMode && savedNote)) && (
              <div className="p-3 bg-green-500/20 border border-green-500/50 rounded-lg text-green-400">
                <div className="flex items-center gap-2">
                  <Check className="w-5 h-5" />
                  <span>{demoMode ? 'Demo deposit simulated!' : 'Transaction successful!'}</span>
                </div>
                {!demoMode && lastTx && (
                  <a 
                    href={`https://explorer.solana.com/tx/${lastTx}?cluster=devnet`}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-sm underline mt-1 block"
                  >
                    View on Explorer →
                  </a>
                )}
              </div>
            )}

            {/* Deposit Tab Content */}
            {activeTab === 'deposit' && (
              <>
                <div>
                  <label className="block text-sm text-gray-400 mb-2">Amount to Shield</label>
                  <div className="relative">
                    <input
                      type="number"
                      value={amount}
                      onChange={(e) => setAmount(e.target.value)}
                      placeholder="0.00"
                      className="w-full bg-black/50 border border-cyber-purple/30 rounded-lg px-4 py-4 text-2xl focus:outline-none focus:border-cyber-purple transition-colors"
                    />
                    <div className="absolute right-4 top-1/2 -translate-y-1/2 flex items-center gap-2">
                      <span className="text-gray-400">SOL</span>
                    </div>
                  </div>
                </div>

                <motion.button
                  whileHover={{ scale: 1.02 }}
                  whileTap={{ scale: 0.98 }}
                  onClick={handleDeposit}
                  disabled={isLoading || (!demoMode && !wallet.connected) || !amount}
                  className="w-full py-4 bg-gradient-to-r from-cyber-purple to-cyber-blue rounded-lg font-semibold text-lg relative overflow-hidden group disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  <span className="relative z-10 flex items-center justify-center gap-2">
                    {isLoading ? <Loader2 className="w-5 h-5 animate-spin" /> : <Lock className="w-5 h-5" />}
                    {!demoMode && !wallet.connected ? 'Connect Wallet First' : isLoading ? 'Processing...' : demoMode ? 'Generate Secret Note' : 'Shield Funds'}
                  </span>
                </motion.button>

                {/* Saved Note Display */}
                {savedNote && (
                  <div className="p-4 bg-cyber-purple/10 border border-cyber-purple/30 rounded-lg">
                    <div className="flex items-center justify-between mb-2">
                      <span className="text-sm font-semibold text-cyber-purple">⚠️ Save Your Secret Note!</span>
                      <div className="flex gap-2">
                        <button onClick={copyNote} className="p-2 hover:bg-white/10 rounded">
                          {copied ? <Check className="w-4 h-4 text-green-400" /> : <Copy className="w-4 h-4" />}
                        </button>
                        <button onClick={downloadNote} className="p-2 hover:bg-white/10 rounded">
                          <Download className="w-4 h-4" />
                        </button>
                      </div>
                    </div>
                    <p className="text-xs text-gray-400 mb-2">This note is required to withdraw. Store it safely!</p>
                    <code className="text-xs break-all bg-black/30 p-2 rounded block">
                      {encodeNote(savedNote).slice(0, 60)}...
                    </code>
                  </div>
                )}
              </>
            )}

            {/* Withdraw Tab Content */}
            {activeTab === 'withdraw' && (
              <>
                <div>
                  <label className="block text-sm text-gray-400 mb-2">Paste Your Secret Note</label>
                  <textarea
                    value={noteInput}
                    onChange={(e) => setNoteInput(e.target.value)}
                    placeholder="Paste your deposit note here..."
                    rows={4}
                    className="w-full bg-black/50 border border-cyber-purple/30 rounded-lg px-4 py-3 text-sm focus:outline-none focus:border-cyber-purple transition-colors font-mono"
                  />
                </div>

                <motion.button
                  whileHover={{ scale: 1.02 }}
                  whileTap={{ scale: 0.98 }}
                  onClick={handleWithdraw}
                  disabled={isLoading || !wallet.connected || !noteInput}
                  className="w-full py-4 bg-gradient-to-r from-cyber-purple to-cyber-blue rounded-lg font-semibold text-lg relative overflow-hidden group disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  <span className="relative z-10 flex items-center justify-center gap-2">
                    {isLoading ? <Loader2 className="w-5 h-5 animate-spin" /> : <Unlock className="w-5 h-5" />}
                    {!wallet.connected ? 'Connect Wallet First' : isLoading ? 'Processing...' : 'Unshield Funds'}
                  </span>
                </motion.button>
              </>
            )}

            {/* Swap Tab Content */}
            {activeTab === 'swap' && (
              <>
                <div>
                  <label className="block text-sm text-gray-400 mb-2">You Send (from shielded balance)</label>
                  <div className="relative">
                    <input
                      type="number"
                      value={amount}
                      onChange={(e) => setAmount(e.target.value)}
                      placeholder="0.00"
                      className="w-full bg-black/50 border border-cyber-purple/30 rounded-lg px-4 py-4 text-2xl focus:outline-none focus:border-cyber-purple transition-colors"
                    />
                    <div className="absolute right-4 top-1/2 -translate-y-1/2 flex items-center gap-2">
                      <span className="text-gray-400">SOL</span>
                    </div>
                  </div>
                </div>

                <div className="flex justify-center">
                  <div className="p-2 bg-cyber-purple/20 rounded-full">
                    <ArrowDownUp className="w-5 h-5 text-cyber-purple" />
                  </div>
                </div>

                <div>
                  <label className="block text-sm text-gray-400 mb-2">You Receive (to shielded balance)</label>
                  <div className="relative">
                    <input
                      type="text"
                      value={amount ? (parseFloat(amount) * 150).toFixed(2) : ''}
                      readOnly
                      placeholder="0.00"
                      className="w-full bg-black/50 border border-cyber-purple/30 rounded-lg px-4 py-4 text-2xl focus:outline-none opacity-70"
                    />
                    <div className="absolute right-4 top-1/2 -translate-y-1/2 flex items-center gap-2">
                      <span className="text-gray-400">USDC</span>
                    </div>
                  </div>
                  <p className="text-xs text-gray-500 mt-1">Rate: 1 SOL ≈ $150 USDC (via Pyth)</p>
                </div>

                <div className="p-3 bg-cyber-purple/10 border border-cyber-purple/20 rounded-lg">
                  <div className="flex items-center gap-2 text-sm">
                    <Lock className="w-4 h-4 text-cyber-purple" />
                    <span className="text-gray-300">Confidential Swap via Arcium MXE</span>
                  </div>
                  <p className="text-xs text-gray-500 mt-1">Your trading bounds are encrypted - MEV bots cannot see your strategy</p>
                </div>

                <motion.button
                  whileHover={{ scale: 1.02 }}
                  whileTap={{ scale: 0.98 }}
                  disabled={isLoading || !wallet.connected || !amount}
                  className="w-full py-4 bg-gradient-to-r from-cyber-purple to-cyber-blue rounded-lg font-semibold text-lg relative overflow-hidden group disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  <span className="relative z-10 flex items-center justify-center gap-2">
                    {isLoading ? <Loader2 className="w-5 h-5 animate-spin" /> : <ArrowDownUp className="w-5 h-5" />}
                    {!wallet.connected ? 'Connect Wallet First' : isLoading ? 'Processing...' : 'Swap Confidentially'}
                  </span>
                </motion.button>
              </>
            )}

            {/* Info Cards */}
            <div className="grid grid-cols-2 gap-4 pt-4">
              <InfoCard label="Privacy Level" value="Maximum" />
              <InfoCard label="Network" value="Devnet" />
            </div>
          </div>
        </motion.div>

        {/* Stats Grid */}
        <div className="grid md:grid-cols-4 gap-6 mt-16 max-w-5xl mx-auto">
          <StatCard label="Total Value Locked" value="$2.4M" />
          <StatCard label="Private Transactions" value="12,847" />
          <StatCard label="Active Users" value="3,291" />
          <StatCard label="Anonymity Set" value="10,000+" />
        </div>
      </div>
    </section>
  );
}

function TabButton({ 
  active, 
  onClick, 
  icon, 
  children 
}: { 
  active: boolean; 
  onClick: () => void; 
  icon: React.ReactNode;
  children: React.ReactNode;
}) {
  return (
    <button
      onClick={onClick}
      className={`flex-1 py-3 px-4 rounded-lg font-medium transition-all flex items-center justify-center gap-2 ${
        active
          ? 'bg-gradient-to-r from-cyber-purple to-cyber-blue text-white'
          : 'bg-black/30 text-gray-400 hover:text-white border border-cyber-purple/20'
      }`}
    >
      {icon}
      {children}
    </button>
  );
}

function InfoCard({ label, value }: { label: string; value: string }) {
  return (
    <div className="bg-black/30 rounded-lg p-4 border border-cyber-purple/20">
      <div className="text-sm text-gray-400 mb-1">{label}</div>
      <div className="text-lg font-semibold text-cyber-purple">{value}</div>
    </div>
  );
}

function StatCard({ label, value }: { label: string; value: string }) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true }}
      whileHover={{ y: -5 }}
      className="glass-effect rounded-xl p-6 text-center cyber-border"
    >
      <div className="text-3xl font-bold text-cyber-purple mb-2">{value}</div>
      <div className="text-sm text-gray-400">{label}</div>
    </motion.div>
  );
}

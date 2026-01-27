'use client';

import { motion } from 'framer-motion';
import { useState } from 'react';
import { ArrowDownUp, Lock, Unlock, Droplets } from 'lucide-react';

export default function PrivacyVault() {
  const [activeTab, setActiveTab] = useState<'deposit' | 'withdraw' | 'swap'>('deposit');
  const [amount, setAmount] = useState('');

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
            {/* Amount Input */}
            <div>
              <label className="block text-sm text-gray-400 mb-2">Amount</label>
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

            {/* Action Button */}
            <motion.button
              whileHover={{ scale: 1.02 }}
              whileTap={{ scale: 0.98 }}
              className="w-full py-4 bg-gradient-to-r from-cyber-purple to-cyber-blue rounded-lg font-semibold text-lg relative overflow-hidden group"
            >
              <span className="relative z-10 flex items-center justify-center gap-2">
                <Lock className="w-5 h-5" />
                {activeTab === 'deposit' && 'Deposit Privately'}
                {activeTab === 'withdraw' && 'Withdraw Anonymously'}
                {activeTab === 'swap' && 'Swap Confidentially'}
              </span>
              <div className="absolute inset-0 bg-gradient-to-r from-cyber-blue to-cyber-purple opacity-0 group-hover:opacity-100 transition-opacity"></div>
            </motion.button>

            {/* Info Cards */}
            <div className="grid grid-cols-2 gap-4 pt-4">
              <InfoCard label="Privacy Level" value="Maximum" />
              <InfoCard label="Network Fee" value="~0.001 SOL" />
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

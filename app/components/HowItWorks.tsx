'use client';

import { motion } from 'framer-motion';
import { Database, Shield, Cpu, Network } from 'lucide-react';

export default function HowItWorks() {
  return (
    <section id="how-it-works" className="relative py-32 px-6">
      <div className="max-w-7xl mx-auto">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          className="text-center mb-20"
        >
          <h2 className="text-5xl font-bold mb-4 bg-gradient-to-r from-cyber-purple to-cyber-blue bg-clip-text text-transparent">
            How It Works
          </h2>
          <p className="text-gray-400 text-lg max-w-2xl mx-auto">
            A three-layer architecture combining zero-knowledge proofs, confidential computation, and decentralized exchange integration.
          </p>
        </motion.div>

        {/* Architecture Diagram */}
        <div className="relative max-w-5xl mx-auto mb-20">
          {/* Connection Lines */}
          <svg className="absolute inset-0 w-full h-full" style={{ zIndex: 0 }}>
            <defs>
              <linearGradient id="lineGradient" x1="0%" y1="0%" x2="100%" y2="0%">
                <stop offset="0%" stopColor="#8B5CF6" stopOpacity="0.5" />
                <stop offset="100%" stopColor="#3B82F6" stopOpacity="0.5" />
              </linearGradient>
            </defs>
            <motion.path
              d="M 150 100 Q 400 50 650 100"
              stroke="url(#lineGradient)"
              strokeWidth="2"
              fill="none"
              initial={{ pathLength: 0 }}
              whileInView={{ pathLength: 1 }}
              viewport={{ once: true }}
              transition={{ duration: 2, ease: "easeInOut" }}
            />
            <motion.path
              d="M 150 250 Q 400 300 650 250"
              stroke="url(#lineGradient)"
              strokeWidth="2"
              fill="none"
              initial={{ pathLength: 0 }}
              whileInView={{ pathLength: 1 }}
              viewport={{ once: true }}
              transition={{ duration: 2, ease: "easeInOut", delay: 0.2 }}
            />
          </svg>

          <div className="grid md:grid-cols-3 gap-8 relative z-10">
            <LayerCard
              icon={<Shield className="w-12 h-12" />}
              title="Layer 1: Privacy"
              description="Zero-knowledge proofs ensure transaction privacy. Merkle trees track commitments without revealing amounts or recipients."
              features={["ZK-SNARKs", "Nullifier System", "Merkle Trees"]}
              delay={0}
            />
            <LayerCard
              icon={<Cpu className="w-12 h-12" />}
              title="Layer 2: Computation"
              description="Arcium FHE/MPC enables confidential computation on encrypted data without exposing sensitive information."
              features={["Encrypted State", "MPC Protocols", "TEE Integration"]}
              delay={0.2}
            />
            <LayerCard
              icon={<Network className="w-12 h-12" />}
              title="Layer 3: DEX"
              description="Jupiter aggregator integration allows private swaps across all Solana DEXs with optimal routing."
              features={["Jupiter V6", "Best Rates", "Private Routing"]}
              delay={0.4}
            />
          </div>
        </div>

        {/* Process Flow */}
        <div className="grid md:grid-cols-4 gap-6">
          <ProcessStep
            number="01"
            title="Deposit"
            description="Lock assets into privacy vault with ZK commitment"
            delay={0}
          />
          <ProcessStep
            number="02"
            title="Encrypt"
            description="FHE layer encrypts transaction data for confidential computation"
            delay={0.2}
          />
          <ProcessStep
            number="03"
            title="Execute"
            description="Perform swaps or transfers on encrypted state"
            delay={0.4}
          />
          <ProcessStep
            number="04"
            title="Withdraw"
            description="Prove ownership with ZK proof and withdraw anonymously"
            delay={0.6}
          />
        </div>
      </div>
    </section>
  );
}

function LayerCard({ 
  icon, 
  title, 
  description, 
  features,
  delay 
}: { 
  icon: React.ReactNode; 
  title: string; 
  description: string;
  features: string[];
  delay: number;
}) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true }}
      transition={{ duration: 0.6, delay }}
      whileHover={{ y: -10 }}
      className="glass-effect rounded-xl p-8 cyber-border group"
    >
      <div className="text-cyber-purple mb-6 group-hover:text-cyber-blue transition-colors">
        {icon}
      </div>
      <h3 className="text-2xl font-bold mb-4 text-glow">{title}</h3>
      <p className="text-gray-400 mb-6">{description}</p>
      <div className="space-y-2">
        {features.map((feature, i) => (
          <div key={i} className="flex items-center gap-2 text-sm">
            <div className="w-1.5 h-1.5 rounded-full bg-cyber-purple"></div>
            <span className="text-gray-300">{feature}</span>
          </div>
        ))}
      </div>
    </motion.div>
  );
}

function ProcessStep({ 
  number, 
  title, 
  description, 
  delay 
}: { 
  number: string; 
  title: string; 
  description: string;
  delay: number;
}) {
  return (
    <motion.div
      initial={{ opacity: 0, x: -20 }}
      whileInView={{ opacity: 1, x: 0 }}
      viewport={{ once: true }}
      transition={{ duration: 0.6, delay }}
      className="relative"
    >
      <div className="glass-effect rounded-xl p-6 cyber-border h-full">
        <div className="text-5xl font-bold text-cyber-purple/30 mb-4">{number}</div>
        <h4 className="text-xl font-semibold mb-2">{title}</h4>
        <p className="text-gray-400 text-sm">{description}</p>
      </div>
      {number !== "04" && (
        <div className="hidden md:block absolute top-1/2 -right-3 w-6 h-0.5 bg-gradient-to-r from-cyber-purple to-transparent"></div>
      )}
    </motion.div>
  );
}

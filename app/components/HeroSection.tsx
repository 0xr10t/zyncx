'use client';

import { motion } from 'framer-motion';
import { Shield, Zap, Lock, ArrowRight } from 'lucide-react';

export default function HeroSection() {
  return (
    <section className="relative min-h-screen flex items-center justify-center overflow-hidden">
      {/* Animated background grid */}
      <div className="absolute inset-0 bg-cyber-grid bg-grid opacity-20"></div>
      
      {/* Scanning line effect */}
      <div className="absolute inset-0 overflow-hidden">
        <div className="absolute w-full h-px bg-gradient-to-r from-transparent via-cyber-purple to-transparent animate-scan"></div>
      </div>

      {/* Floating orbs */}
      <motion.div
        className="absolute top-1/4 left-1/4 w-64 h-64 bg-cyber-purple rounded-full blur-3xl opacity-20"
        animate={{
          scale: [1, 1.2, 1],
          opacity: [0.2, 0.3, 0.2],
        }}
        transition={{
          duration: 8,
          repeat: Infinity,
          ease: "easeInOut",
        }}
      />
      <motion.div
        className="absolute bottom-1/4 right-1/4 w-96 h-96 bg-cyber-blue rounded-full blur-3xl opacity-20"
        animate={{
          scale: [1, 1.3, 1],
          opacity: [0.2, 0.25, 0.2],
        }}
        transition={{
          duration: 10,
          repeat: Infinity,
          ease: "easeInOut",
        }}
      />

      <div className="relative z-10 max-w-7xl mx-auto px-6 text-center">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8 }}
        >
          <h1 className="text-7xl md:text-8xl font-bold mb-6 bg-gradient-to-r from-cyber-purple via-cyber-blue to-cyber-cyan bg-clip-text text-transparent">
            ZYNCX
          </h1>
          <p className="text-2xl md:text-3xl font-light mb-4 text-gray-300">
            Privacy Protocol. <span className="text-glow text-cyber-purple">Turbocharged.</span>
          </p>
          <p className="text-lg md:text-xl text-gray-400 max-w-3xl mx-auto mb-12">
            Zero-knowledge privacy meets confidential computation on Solana. 
            Deposit, swap, and withdraw with complete anonymity.
          </p>

          <div className="flex flex-col sm:flex-row gap-6 justify-center items-center mb-16">
            <motion.button
              whileHover={{ scale: 1.05 }}
              whileTap={{ scale: 0.95 }}
              className="group relative px-8 py-4 bg-gradient-to-r from-cyber-purple to-cyber-blue rounded-lg font-semibold text-lg overflow-hidden"
            >
              <span className="relative z-10 flex items-center gap-2">
                Launch App
                <ArrowRight className="w-5 h-5 group-hover:translate-x-1 transition-transform" />
              </span>
              <div className="absolute inset-0 bg-gradient-to-r from-cyber-blue to-cyber-purple opacity-0 group-hover:opacity-100 transition-opacity"></div>
            </motion.button>
            
            <motion.button
              whileHover={{ scale: 1.05 }}
              whileTap={{ scale: 0.95 }}
              className="px-8 py-4 border-2 border-cyber-purple rounded-lg font-semibold text-lg cyber-border"
            >
              Read Docs
            </motion.button>
          </div>

          {/* Feature cards */}
          <div className="grid md:grid-cols-3 gap-6 max-w-5xl mx-auto">
            <FeatureCard
              icon={<Shield className="w-8 h-8" />}
              title="Zero-Knowledge Proofs"
              description="Cryptographic privacy guarantees using ZK-SNARKs"
              delay={0.2}
            />
            <FeatureCard
              icon={<Lock className="w-8 h-8" />}
              title="Confidential Swaps"
              description="Private DEX integration with Jupiter aggregator"
              delay={0.4}
            />
            <FeatureCard
              icon={<Zap className="w-8 h-8" />}
              title="FHE Computing"
              description="Arcium-powered encrypted computation layer"
              delay={0.6}
            />
          </div>
        </motion.div>
      </div>
    </section>
  );
}

function FeatureCard({ icon, title, description, delay }: { 
  icon: React.ReactNode; 
  title: string; 
  description: string;
  delay: number;
}) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.6, delay }}
      whileHover={{ y: -5 }}
      className="glass-effect rounded-xl p-6 cyber-border group"
    >
      <div className="text-cyber-purple mb-4 group-hover:text-cyber-blue transition-colors">
        {icon}
      </div>
      <h3 className="text-xl font-semibold mb-2 text-glow">{title}</h3>
      <p className="text-gray-400 text-sm">{description}</p>
    </motion.div>
  );
}

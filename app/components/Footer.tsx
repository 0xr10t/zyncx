'use client';

import { Github, Twitter, FileText, MessageCircle } from 'lucide-react';

export default function Footer() {
  return (
    <footer className="relative border-t border-cyber-purple/20 py-12 px-6">
      <div className="max-w-7xl mx-auto">
        <div className="grid md:grid-cols-4 gap-8 mb-8">
          {/* Brand */}
          <div>
            <h3 className="text-2xl font-bold bg-gradient-to-r from-cyber-purple to-cyber-blue bg-clip-text text-transparent mb-4">
              ZYNCX
            </h3>
            <p className="text-gray-400 text-sm">
              Privacy-first DeFi protocol on Solana. Built with zero-knowledge proofs and confidential computation.
            </p>
          </div>

          {/* Links */}
          <div>
            <h4 className="font-semibold mb-4 text-cyber-purple">Protocol</h4>
            <ul className="space-y-2 text-sm text-gray-400">
              <li><a href="#" className="hover:text-cyber-purple transition-colors">Documentation</a></li>
              <li><a href="#" className="hover:text-cyber-purple transition-colors">Whitepaper</a></li>
              <li><a href="#" className="hover:text-cyber-purple transition-colors">Audit Reports</a></li>
              <li><a href="#" className="hover:text-cyber-purple transition-colors">GitHub</a></li>
            </ul>
          </div>

          <div>
            <h4 className="font-semibold mb-4 text-cyber-purple">Developers</h4>
            <ul className="space-y-2 text-sm text-gray-400">
              <li><a href="#" className="hover:text-cyber-purple transition-colors">SDK</a></li>
              <li><a href="#" className="hover:text-cyber-purple transition-colors">API Reference</a></li>
              <li><a href="#" className="hover:text-cyber-purple transition-colors">Smart Contracts</a></li>
              <li><a href="#" className="hover:text-cyber-purple transition-colors">Integration Guide</a></li>
            </ul>
          </div>

          <div>
            <h4 className="font-semibold mb-4 text-cyber-purple">Community</h4>
            <ul className="space-y-2 text-sm text-gray-400">
              <li><a href="#" className="hover:text-cyber-purple transition-colors">Discord</a></li>
              <li><a href="#" className="hover:text-cyber-purple transition-colors">Twitter</a></li>
              <li><a href="#" className="hover:text-cyber-purple transition-colors">Forum</a></li>
              <li><a href="#" className="hover:text-cyber-purple transition-colors">Blog</a></li>
            </ul>
          </div>
        </div>

        {/* Social Links */}
        <div className="flex items-center justify-between pt-8 border-t border-cyber-purple/20">
          <p className="text-gray-400 text-sm">
            © 2026 ZYNCX. Built for privacy maximalists.
          </p>
          <div className="flex items-center gap-4">
            <SocialLink icon={<Github className="w-5 h-5" />} href="#" />
            <SocialLink icon={<Twitter className="w-5 h-5" />} href="#" />
            <SocialLink icon={<FileText className="w-5 h-5" />} href="#" />
            <SocialLink icon={<MessageCircle className="w-5 h-5" />} href="#" />
          </div>
        </div>
      </div>
    </footer>
  );
}

function SocialLink({ icon, href }: { icon: React.ReactNode; href: string }) {
  return (
    <a
      href={href}
      className="text-gray-400 hover:text-cyber-purple transition-colors p-2 rounded-lg hover:bg-cyber-purple/10"
    >
      {icon}
    </a>
  );
}

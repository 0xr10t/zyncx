import Navbar from '@/components/Navbar';
import HeroSection from '@/components/HeroSection';
import PrivacyVault from '@/components/PrivacyVault';
import HowItWorks from '@/components/HowItWorks';
import Footer from '@/components/Footer';

export default function Home() {
  return (
    <div className="min-h-screen">
      <Navbar />
      <HeroSection />
      <PrivacyVault />
      <HowItWorks />
      <Footer />
    </div>
  );
}

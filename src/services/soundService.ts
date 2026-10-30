/**
 * Service handling pleasant, lightweight audio feedback for chat events (SRP).
 * Uses Web Audio API for zero-latency, asset-free bubble pop synthesis.
 */
class SoundService {
  private audioCtx: AudioContext | null = null;
  private lastPlayTime = 0;
  private throttleMs = 1500;

  private getAudioContext(): AudioContext | null {
    if (typeof window === "undefined") return null;
    if (!this.audioCtx) {
      const AudioContextClass =
        window.AudioContext ||
        (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext;
      if (AudioContextClass) {
        this.audioCtx = new AudioContextClass();
      }
    }
    if (this.audioCtx && this.audioCtx.state === "suspended") {
      this.audioCtx.resume().catch(() => {});
    }
    return this.audioCtx;
  }

  /**
   * Plays a cheerful, gentle bubble pop sound with anti-spam throttling.
   */
  public playBubblePop(): void {
    const now = Date.now();
    if (now - this.lastPlayTime < this.throttleMs) {
      return;
    }
    this.lastPlayTime = now;

    try {
      const ctx = this.getAudioContext();
      if (!ctx) return;

      const osc = ctx.createOscillator();
      const gain = ctx.createGain();

      const startTime = ctx.currentTime;
      const duration = 0.09;

      // Soft bubble "pop" frequency sweep: starts around 420Hz and sweeps to 860Hz
      osc.type = "sine";
      osc.frequency.setValueAtTime(420, startTime);
      osc.frequency.exponentialRampToValueAtTime(860, startTime + duration * 0.7);

      // Volume envelope: rapid attack, smooth exponential decay
      gain.gain.setValueAtTime(0.001, startTime);
      gain.gain.linearRampToValueAtTime(0.22, startTime + 0.012);
      gain.gain.exponentialRampToValueAtTime(0.001, startTime + duration);

      osc.connect(gain);
      gain.connect(ctx.destination);

      osc.start(startTime);
      osc.stop(startTime + duration);
    } catch (e) {
      console.warn("[SoundService] Failed to play bubble pop:", e);
    }
  }
}

export const soundService = new SoundService();

const notes = [
  { frequency: 880, start: 0 },
  { frequency: 1320, start: 0.14 },
];

const length = 0.3;
const volume = 0.2;
let context: AudioContext | undefined;

export async function chime() {
  context ??= new AudioContext();
  await context.resume();

  const now = context.currentTime;
  for (const { frequency, start } of notes) {
    const oscillator = context.createOscillator();
    const gain = context.createGain();
    oscillator.frequency.value = frequency;
    gain.gain.setValueAtTime(volume, now + start);
    gain.gain.exponentialRampToValueAtTime(0.001, now + start + length);
    oscillator.connect(gain).connect(context.destination);
    oscillator.start(now + start);
    oscillator.stop(now + start + length);
  }
}

// Mascot source lives in assets/mascots. Vite inlines the SVG at build time.
import cortex from '../../../../assets/mascots/cortex.svg?raw';
import axon from '../../../../assets/mascots/axon.svg?raw';
import thalamus from '../../../../assets/mascots/thalamus.svg?raw';
import synapse from '../../../../assets/mascots/synapse.svg?raw';
import soma from '../../../../assets/mascots/soma.svg?raw';
import hippocampus from '../../../../assets/mascots/hippocampus.svg?raw';
import retina from '../../../../assets/mascots/retina.svg?raw';
import aesthesis from '../../../../assets/mascots/aesthesis.svg?raw';

const SOURCES: Record<string, string> = { cortex, axon, thalamus, synapse, soma, hippocampus, retina, aesthesis };

export function mascot(id: string, size = 24): string {
  const svg = SOURCES[id.toLowerCase()];
  if (!svg) return '';
  return svg.replace('<svg ', `<svg width="${size}" height="${size}" aria-hidden="true" focusable="false" `);
}

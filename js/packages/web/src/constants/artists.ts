import { PublicKey } from '@solana/web3.js';

// Dummy lookup file until we integrate name service
export interface IArtist {
  address: PublicKey;
  name: string;
}

export const ARTISTS: IArtist[] = [
  {
    address: new PublicKey('EDshWM3jBy2YUszMiFLAFLx3WkbtqR9An7JZzvg22R1P'),
    name: 'Jordan Prince',
  },
  {
    address: new PublicKey('HRjJmdknBEzMXFUwF3mcpbQ3PrwrC5svVSGfXLpRDmUR'),
    name: 'Bartosz Lipinski',
  },
  {
    address: new PublicKey('3yp9iTsCgZoBsXhtRLB8cWHNcTDeR6VJniRuDrHNTuxU'),
    name: 'Helen Azgolei',
  },
];

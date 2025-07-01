import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { Channel } from '../components/ChannelList';
import type Hls from 'hls.js';

export interface PlayerState {
  // Player refs (will be set from components)
  videoRef: React.RefObject<HTMLVideoElement> | null;
  hlsRef: React.MutableRefObject<Hls | null> | null;
  
  // Player state
  isPlaying: boolean;
  currentChannel: Channel | null;
  volume: number;
  muted: boolean;
}

export interface PlayerActions {
  // Ref management
  setVideoRef: (ref: React.RefObject<HTMLVideoElement>) => void;
  setHlsRef: (ref: React.MutableRefObject<Hls | null>) => void;
  
  // Playback control
  playChannel: (channel: Channel) => Promise<void>;
  stopPlayback: () => void;
  setVolume: (volume: number) => void;
  setMuted: (muted: boolean) => void;
  
  // External player
  playInMpv: (channel: Channel) => Promise<void>;
  
  // Utility
  reset: () => void;
}

type PlayerStore = PlayerState & PlayerActions;

export const usePlayerStore = create<PlayerStore>((set, get) => ({
  // Initial state
  videoRef: null,
  hlsRef: null,
  isPlaying: false,
  currentChannel: null,
  volume: 1.0,
  muted: false,

  // Actions
  setVideoRef: (ref: React.RefObject<HTMLVideoElement>) => set({ videoRef: ref }),
  
  setHlsRef: (ref: React.MutableRefObject<Hls | null>) => set({ hlsRef: ref }),
  
  playChannel: async (channel: Channel) => {
    // This method sets up the channel for playback
    // The actual HLS setup will still be handled in the component
    // due to the complexity of managing DOM refs in Zustand
    set({ 
      currentChannel: channel,
      isPlaying: true 
    });
  },
  
  stopPlayback: () => {
    const { videoRef, hlsRef } = get();
    
    if (videoRef?.current) {
      videoRef.current.pause();
      videoRef.current.src = '';
    }
    
    if (hlsRef?.current) {
      hlsRef.current.destroy();
    }
    
    set({ 
      isPlaying: false,
      currentChannel: null 
    });
  },
  
  setVolume: (volume: number) => {
    const { videoRef } = get();
    
    if (videoRef?.current) {
      videoRef.current.volume = volume;
    }
    
    set({ volume });
  },
  
  setMuted: (muted: boolean) => {
    const { videoRef } = get();
    
    if (videoRef?.current) {
      videoRef.current.muted = muted;
    }
    
    set({ muted });
  },
  
  playInMpv: async (channel: Channel) => {
    try {
      // Stop current web playback
      const { stopPlayback } = get();
      stopPlayback();
      
      // Play in MPV
      await invoke('play_channel', { channel });
      
      // Note: History update should be handled by the component/app level
      console.log(`Playing ${channel.name} in MPV`);
    } catch (error) {
      console.error('Failed to play in MPV:', error);
    }
  },
  
  reset: () => set({
    videoRef: null,
    hlsRef: null,
    isPlaying: false,
    currentChannel: null,
    volume: 1.0,
    muted: false,
  }),
}));
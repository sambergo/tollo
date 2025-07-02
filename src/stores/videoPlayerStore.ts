import { create } from 'zustand';
import { RefObject } from 'react';
import Hls from 'hls.js';

interface VideoPlayerState {
  // State
  videoRef: RefObject<HTMLVideoElement> | null;
  hlsInstance: Hls | null;
  
  // Actions
  setVideoRef: (ref: RefObject<HTMLVideoElement>) => void;
  setHlsInstance: (hls: Hls | null) => void;
  cleanupVideo: () => void;
}

const useVideoPlayerStore = create<VideoPlayerState>((set, get) => ({
  // Initial state
  videoRef: null,
  hlsInstance: null,
  
  // Actions
  setVideoRef: (ref) => set({ videoRef: ref }),
  
  setHlsInstance: (hls) => {
    // Cleanup previous instance if exists
    const { hlsInstance } = get();
    if (hlsInstance) {
      hlsInstance.destroy();
    }
    set({ hlsInstance: hls });
  },
  
  cleanupVideo: () => {
    const { videoRef, hlsInstance } = get();
    
    if (videoRef?.current) {
      videoRef.current.pause();
      videoRef.current.src = '';
    }
    
    if (hlsInstance) {
      hlsInstance.destroy();
      set({ hlsInstance: null });
    }
  },
}));

export default useVideoPlayerStore;
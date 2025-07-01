# Zustand Stores - Developer Guide

This directory contains all Zustand state management stores for the application.

## Quick Start

### 1. Using Individual Stores

```tsx
import { useChannelStore, useFavoritesStore } from '../stores';

function MyComponent() {
  // Get specific state and actions
  const channels = useChannelStore(state => state.channels);
  const isLoading = useChannelStore(state => state.isLoadingChannelList);
  const fetchChannels = useChannelStore(state => state.fetchChannels);
  
  const favorites = useFavoritesStore(state => state.favorites);
  const toggleFavorite = useFavoritesStore(state => state.toggleFavorite);
  
  // Use in component...
}
```

### 2. Using the Orchestrated Hook (Recommended)

```tsx
import { useAppState } from '../hooks/useAppState';

function MyComponent() {
  const {
    // State
    channels,
    favorites,
    selectedChannel,
    isLoadingChannelList,
    
    // Actions
    handleToggleFavorite,
    handleSelectChannel,
    setActiveTab
  } = useAppState();
  
  // All state and actions available in one place
}
```

## Store Overview

| Store | Purpose | Key State | Key Actions |
|-------|---------|-----------|-------------|
| **Channel** | Channel data & search | `channels`, `searchQuery`, `selectedChannelListId` | `fetchChannels()`, `performSearch()` |
| **Group** | Group filtering | `groups`, `enabledGroups`, `groupDisplayMode` | `toggleGroupEnabled()`, `setGroupDisplayMode()` |
| **Favorites** | Favorite channels | `favorites` | `toggleFavorite()`, `isFavorite()` |
| **History** | View history | `history` | `fetchHistory()`, `addToHistory()` |
| **UI** | Interface state | `activeTab`, `selectedChannel`, `focusedIndex` | `setActiveTab()`, `setSelectedChannel()` |
| **Filter** | Saved filters | `savedFilters` | `saveFilter()`, `deleteFilter()` |
| **Player** | Video player | `isPlaying`, `currentChannel` | `playChannel()`, `playInMpv()` |

## Common Patterns

### Async Operations
```tsx
// In a component
const fetchChannels = useChannelStore(state => state.fetchChannels);

useEffect(() => {
  fetchChannels(channelListId);
}, [channelListId, fetchChannels]);
```

### State Subscriptions
```tsx
// Subscribe to specific state changes
const isLoading = useChannelStore(state => state.isLoadingChannelList);

// Only re-renders when isLoadingChannelList changes
```

### Actions with Parameters
```tsx
const toggleGroupEnabled = useGroupStore(state => state.toggleGroupEnabled);
const selectedChannelListId = useChannelStore(state => state.selectedChannelListId);

const handleToggle = useCallback((groupName: string) => {
  if (selectedChannelListId) {
    toggleGroupEnabled(groupName, selectedChannelListId);
  }
}, [toggleGroupEnabled, selectedChannelListId]);
```

## Best Practices

### ✅ Do
- Use `useAppState()` for most components (simpler)
- Use individual stores for specific, focused components
- Keep actions simple and focused
- Use TypeScript interfaces for type safety
- Use `useCallback` for action handlers in components

### ❌ Don't
- Mix local state with store state for the same data
- Call store actions inside render functions
- Forget to handle loading states
- Modify store state directly (always use actions)

## Migration from Local State

### Before (Local State)
```tsx
function App() {
  const [channels, setChannels] = useState<Channel[]>([]);
  const [favorites, setFavorites] = useState<Channel[]>([]);
  
  const fetchChannels = async () => {
    const data = await invoke('get_channels');
    setChannels(data);
  };
  
  return (
    <ChildComponent 
      channels={channels}
      favorites={favorites}
      onFetchChannels={fetchChannels}
    />
  );
}
```

### After (Zustand)
```tsx
function App() {
  const { channels, favorites, fetchChannels } = useAppState();
  
  return (
    <ChildComponent />  // No props needed!
  );
}

function ChildComponent() {
  const { channels, favorites, fetchChannels } = useAppState();
  // Direct access to state and actions
}
```

## Performance Tips

### Selective Subscriptions
```tsx
// Only subscribes to channels, not the entire store
const channels = useChannelStore(state => state.channels);

// More specific subscriptions for better performance
const channelCount = useChannelStore(state => state.channels.length);
```

### Computed Values
```tsx
// Use the computed values from useAppState
const { filteredChannels, listItems } = useAppState();

// These are memoized and only recompute when dependencies change
```

## Debugging

### DevTools
Zustand works with Redux DevTools for debugging:
```tsx
import { devtools } from 'zustand/middleware';

export const useChannelStore = create<ChannelStore>()(
  devtools(
    (set, get) => ({
      // store implementation
    }),
    {
      name: 'channel-store', // Name in DevTools
    }
  )
);
```

### Logging
Add console logs to actions for debugging:
```tsx
fetchChannels: async (id = null) => {
  console.log('Fetching channels for list:', id);
  try {
    const fetchedChannels = await invoke<Channel[]>('get_channels', { id });
    console.log('Fetched channels:', fetchedChannels.length);
    set({ channels: fetchedChannels });
  } catch (error) {
    console.error('Failed to fetch channels:', error);
  }
},
```

## Store Files

- `index.ts` - Main exports
- `channelStore.ts` - Channel data and search
- `groupStore.ts` - Group filtering and display
- `favoritesStore.ts` - Favorite channels
- `historyStore.ts` - Viewing history  
- `uiStore.ts` - UI state management
- `filterStore.ts` - Saved filters
- `playerStore.ts` - Video player state
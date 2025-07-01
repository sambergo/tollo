# Zustand State Management Migration

This document outlines the implementation of Zustand state management in the IPTV React application, replacing the previous prop drilling approach with centralized stores.

## Overview

The migration introduces 7 domain-specific Zustand stores that manage different aspects of the application state:

1. **Channel Store** - Manages channels, search, and channel list operations
2. **Group Store** - Manages groups, group filtering, and display modes
3. **Favorites Store** - Manages favorite channels
4. **History Store** - Manages viewing history
5. **UI Store** - Manages user interface state (tabs, focus, selection)
6. **Filter Store** - Manages saved filters
7. **Player Store** - Manages video player state

## Architecture

### Store Structure

Each store follows a consistent pattern:
- **State Interface**: Defines the state shape
- **Actions Interface**: Defines available actions
- **Store Implementation**: Uses Zustand's `create` function

```typescript
// Example pattern used in all stores
export interface ExampleState {
  data: SomeType[];
  loading: boolean;
}

export interface ExampleActions {
  setData: (data: SomeType[]) => void;
  fetchData: () => Promise<void>;
  reset: () => void;
}

export const useExampleStore = create<ExampleState & ExampleActions>((set, get) => ({
  // Initial state
  data: [],
  loading: false,
  
  // Actions
  setData: (data) => set({ data }),
  fetchData: async () => {
    // Implementation
  },
  reset: () => set({ data: [], loading: false })
}));
```

### Central Orchestration

The `useAppState` hook orchestrates all stores and provides:
- Coordinated state management across stores
- Complex business logic (like channel list loading)
- Computed values (like filtered channels)
- Action handlers for components
- Effects for side-effects and data synchronization

## Store Details

### 1. Channel Store (`src/stores/channelStore.ts`)

**Purpose**: Manages all channel-related state and operations.

**Key State**:
- `channels`: Array of channel data
- `selectedChannelListId`: Currently selected channel list
- `searchQuery` & `debouncedSearchQuery`: Search functionality
- `isLoadingChannelList`: Loading state for channel list operations

**Key Actions**:
- `fetchChannels()`: Fetches channels from backend
- `searchChannels()`: Performs channel search
- `performSearch()`: Performs search and updates channels
- `setSelectedChannelListId()`: Changes active channel list

### 2. Group Store (`src/stores/groupStore.ts`)

**Purpose**: Manages group filtering and display modes.

**Key State**:
- `groups`: Available groups
- `enabledGroups`: Set of enabled groups for filtering
- `selectedGroup`: Currently selected group
- `groupDisplayMode`: Display mode (EnabledGroups or AllGroups)

**Key Actions**:
- `toggleGroupEnabled()`: Enable/disable group
- `selectAllGroups()` / `unselectAllGroups()`: Bulk operations
- `setGroupDisplayMode()`: Change display mode
- `syncGroupsForChannelList()`: Sync groups with backend

### 3. Favorites Store (`src/stores/favoritesStore.ts`)

**Purpose**: Manages favorite channels.

**Key State**:
- `favorites`: Array of favorite channels

**Key Actions**:
- `fetchFavorites()`: Load favorites from backend
- `toggleFavorite()`: Add/remove favorite
- `isFavorite()`: Check if channel is favorite

### 4. History Store (`src/stores/historyStore.ts`)

**Purpose**: Manages viewing history.

**Key State**:
- `history`: Array of recently viewed channels

**Key Actions**:
- `fetchHistory()`: Load history from backend
- `addToHistory()`: Add channel to history
- `clearHistory()`: Clear all history

### 5. UI Store (`src/stores/uiStore.ts`)

**Purpose**: Manages user interface state.

**Key State**:
- `activeTab`: Current active tab
- `focusedIndex`: Currently focused item index
- `selectedChannel`: Currently selected channel

**Key Actions**:
- `setActiveTab()`: Change active tab
- `setFocusedIndex()`: Update focused item
- `setSelectedChannel()`: Select channel

### 6. Filter Store (`src/stores/filterStore.ts`)

**Purpose**: Manages saved filters functionality.

**Key State**:
- `savedFilters`: Array of saved filter configurations
- `isLoading`: Loading state for filter operations

**Key Actions**:
- `fetchSavedFilters()`: Load saved filters
- `saveFilter()`: Save new filter
- `deleteFilter()`: Delete filter
- `refreshFilters()`: Reload filters

### 7. Player Store (`src/stores/playerStore.ts`)

**Purpose**: Manages video player state and operations.

**Key State**:
- `videoRef` & `hlsRef`: References to video elements (set from components)
- `isPlaying`: Playback state
- `currentChannel`: Currently playing channel
- `volume` & `muted`: Audio settings

**Key Actions**:
- `playChannel()`: Start channel playback
- `stopPlayback()`: Stop playback and cleanup
- `playInMpv()`: Play in external MPV player
- `setVolume()` / `setMuted()`: Audio controls

## Benefits of the Migration

### 1. **Eliminated Prop Drilling**
- No more passing props through multiple component layers
- Direct access to state where needed
- Cleaner component interfaces

### 2. **Centralized State Management**
- All state in predictable locations
- Easier debugging and development
- Better state persistence possibilities

### 3. **Improved Performance**
- Components only re-render when their specific state changes
- Better React rendering optimization
- Reduced unnecessary re-renders

### 4. **Better Code Organization**
- Domain-specific stores
- Clear separation of concerns
- Easier to locate and modify functionality

### 5. **Enhanced Maintainability**
- Smaller, focused components
- Reusable store logic
- Easier testing of individual stores

### 6. **Type Safety**
- Full TypeScript support
- Strongly typed actions and state
- Better IDE support and autocomplete

## Usage Examples

### Using Individual Stores

```tsx
import { useChannelStore, useFavoritesStore } from '../stores';

function MyComponent() {
  const channels = useChannelStore(state => state.channels);
  const fetchChannels = useChannelStore(state => state.fetchChannels);
  const toggleFavorite = useFavoritesStore(state => state.toggleFavorite);
  
  // Component logic...
}
```

### Using Orchestrated State

```tsx
import { useAppState } from '../hooks/useAppState';

function MyComponent() {
  const {
    channels,
    favorites,
    handleToggleFavorite,
    handleSelectChannel
  } = useAppState();
  
  // All state and actions available in one place
}
```

## Migration Notes

### What Changed
- **App.tsx**: Reduced from 511 lines to ~160 lines
- **State Management**: Moved from local state to Zustand stores
- **Props**: Significantly reduced prop passing
- **Architecture**: More modular and maintainable

### What Stayed the Same
- **All existing functionality** works exactly as before
- **Component interfaces** mostly unchanged (same props)
- **User experience** is identical
- **Backend API calls** unchanged
- **TypeScript types** preserved

### Files Added
- `src/stores/` - All Zustand store files
- `src/hooks/useAppState.ts` - Central orchestration hook
- `src/App.original.tsx` - Backup of original App.tsx

### Backward Compatibility
The migration maintains 100% backward compatibility:
- All existing components work without modification
- All hooks continue to function
- No breaking changes to component APIs
- Original App.tsx backed up as `App.original.tsx`

## Future Enhancements

### Possible Improvements
1. **Persistence**: Add state persistence using Zustand's persist middleware
2. **DevTools**: Integrate with Redux DevTools for debugging
3. **Subscriptions**: Add selective subscriptions for better performance
4. **Middleware**: Add logging, error handling, or analytics middleware
5. **Store Composition**: Combine related stores for complex operations

### Hook Migration
Existing custom hooks can be gradually migrated to use stores:
- `useChannelSearch` → Integrated into Channel Store
- `useSavedFilters` → Replaced by Filter Store
- `useImageCache` → Could be migrated to a Cache Store
- `useKeyboardNavigation` → Continue using (UI interaction logic)

## Testing the Migration

### Verification Steps
1. **Functionality**: All features work as before
2. **Performance**: No performance regressions
3. **State Management**: State updates correctly across components
4. **Error Handling**: Errors handled gracefully
5. **TypeScript**: No type errors

### Common Issues and Solutions
- **Missing Dependencies**: Ensure Zustand ^5.0.6 is installed
- **Type Errors**: Check store interfaces match usage
- **State Not Updating**: Verify actions are called correctly
- **Performance Issues**: Use selective subscriptions if needed

## Conclusion

This Zustand migration successfully:
- ✅ Centralizes state management
- ✅ Eliminates prop drilling
- ✅ Maintains all existing functionality
- ✅ Improves code maintainability
- ✅ Preserves TypeScript safety
- ✅ Reduces App.tsx complexity by 70%

The application now has a modern, scalable state management architecture that will be easier to maintain and extend going forward.
# Töllö IPTV Player - Keybindings Reference

## Current Keybindings

### Navigation

- `K` - Previous tab
- `J` / `Tab` - Next tab
- `j` / `↓` - Navigate down in lists
- `k` / `↑` - Navigate up in lists
- `l` / `→` - Select item/channel
- `Enter` / `o` - Play in external player

### Enhanced Navigation

- `g` - Go to first item in current page
- `G` - Go to last item in current page
- `Ctrl+u` / `PageUp` - Page up (scroll up by 10 items)
- `Ctrl+d` / `PageDown` - Page down (scroll down by 10 items)
- `Home` - Go to first item in current page
- `End` - Go to last item in current page

### Search & Filtering

- `/` / `i` - Focus search input
- `Escape` - Unfocus search field (if focused) or clear search (if not focused)
- `d` - Clear search
- `D` - Clear all filters (search + group)

### Channel Actions

- `f` - Toggle favorite (in channels tab)

### Saved Filters

- `0-9` - Apply saved filter from slot (0-9)
- `Alt+0-9` - Save current search + group filter to slot (0-9)

### Channel List Management

- `R` - Refresh current channel list

### Group Management

- `A` - Select all groups
- `U` - Unselect all groups
- `t` - Toggle group display mode (enabled/all)
- `Space` - Toggle current group selection (in groups tab)

### Video & Preview

- `m` - Toggle mute
- `F` - Toggle fullscreen (if preview enabled)

---

## Suggested Extended Keybindings

### Direct Tab Navigation

- `1` - Go to Channels tab
- `2` - Go to Favorites tab
- `3` - Go to Groups tab
- `4` - Go to History tab
- `5` - Go to Settings tab

### Channel List Management

- `r` - Refresh current channel list (alternative to R)
- `a` - Add new channel list
- `d` - Delete current channel list
- `e` - Edit current channel list

### Group Management

- `Ctrl+g` - Clear group filter

### Video & Preview

- `v` - Toggle video preview on/off
- `Ctrl+p` - Toggle video controls visibility
- `Ctrl+a` - Toggle autoplay

### Favorites & History

- `F` - Toggle favorite (works in any tab with channels)
- `Delete` - Remove from favorites (in favorites tab)
- `Backspace` - Remove from history (in history tab)
- `Ctrl+h` - Clear all history
- `Ctrl+f` - Clear all favorites

### Settings Quick Access

- `s` - Open settings
- `sp` - Go to Player settings
- `sc` - Go to Cache settings
- `sl` - Go to Channel Lists settings
- `sf` - Go to Saved Filters settings

### Advanced Actions

- `i` - Show channel info/details
- `o` - Open channel in external player
- `y` - Copy channel URL to clipboard
- `w` - Save current channel as favorite
- `b` - Go back to previous tab
- `q` - Quit application
- `?` - Show help/keybindings

### Pagination

- `Page Down` - Next page
- `Page Up` - Previous page
- `Ctrl+n` - Next page
- `Ctrl+b` - Previous page

### Cache Management

- `Ctrl+r` - Refresh image cache
- `Ctrl+Shift+r` - Clear all caches
- `Ctrl+i` - Show cache info

---

## Functionality Categories

### 🎯 **Core Navigation**

- **Tabs**: Channels, Favorites, Groups, History, Settings
- **List Navigation**: Up/down movement, first/last item
- **Pagination**: Page-based navigation for large lists
- **Search Navigation**: Navigate through search results

### 📺 **Channel Management**

- **Playback**: Play in external player (MPV)
- **Favorites**: Add/remove from favorites
- **History**: Track viewed channels
- **Details**: View channel information and metadata
- **Copy**: Copy channel URLs for external use

### 🔍 **Search & Filtering**

- **Real-time Search**: Search across channel names and metadata
- **Saved Filters**: Save and apply search + group combinations
- **Group Filtering**: Filter channels by groups
- **Filter Management**: Clear, modify, and organize filters

### 📝 **Playlist Management**

- **Add Lists**: Add M3U playlists from URLs or files
- **Edit Lists**: Modify existing playlist sources
- **Delete Lists**: Remove playlist sources
- **Refresh**: Update playlist data
- **Default Lists**: Set preferred playlist sources

### 🎬 **Video Preview**

- **In-app Preview**: HLS.js-based video preview
- **Controls**: Show/hide video controls
- **Mute**: Audio control for previews
- **Autoplay**: Automatic playback settings
- **External Player**: Launch in MPV or other players

### 🏷️ **Group Management**

- **Display Modes**: Enabled groups vs all groups
- **Multi-select**: Select multiple groups simultaneously
- **Group Actions**: Enable/disable, select all/none
- **Group Search**: Search within group names

### ⚙️ **Settings & Configuration**

- **Player Settings**: External player configuration
- **Cache Settings**: Image cache management
- **Channel Lists**: Manage playlist sources
- **Saved Filters**: Organize saved search filters
- **UI Preferences**: Preview, controls, autoplay settings

### 💾 **Data Management**

- **Database**: SQLite storage for all data
- **Caching**: Smart caching for performance
- **Persistence**: Save user preferences and state
- **Async Operations**: Background data loading

### 🖼️ **Image Caching**

- **Auto-caching**: Automatic channel logo caching
- **Size Management**: Control cache size and duration
- **Preloading**: Optimize loading performance
- **Cache Clearing**: Manual cache management

### ⌨️ **Keyboard Navigation**

- **Vim-like**: h/j/k/l navigation
- **Arrow Keys**: Alternative navigation
- **Modifiers**: Ctrl/Alt/Shift combinations
- **Context-aware**: Different actions per tab

---

## Implementation Notes

### Priority Levels

- **High**: Essential navigation and playback features
- **Medium**: Convenience features and shortcuts
- **Low**: Advanced features and power-user shortcuts

### Modifier Key Usage

- **Ctrl**: System-level actions (clear, refresh, quit)
- **Alt**: Save operations (save filters)
- **Shift**: Extended actions (shift+delete for permanent removal)
- **No modifier**: Basic navigation and actions

### Context Sensitivity

- Some keys work only in specific tabs (e.g., `f` for favorites in channels)
- Input fields should not trigger navigation keys
- Modal dialogs should have their own key handling

### Extensibility

- The current keyboard navigation system is well-designed for extensions
- New keybindings can be added to the existing `useKeyboardNavigation` hook
- State management through Zustand stores makes it easy to add new actions

---

## Recommended Implementation Order

1. **Enhanced Navigation** (g, G, Ctrl+u, Ctrl+d)
2. **Direct Tab Navigation** (1-5)
3. **Search Improvements** (/, Escape, Ctrl+l)
4. **Video Controls** (v, m, Ctrl+p)
5. **Group Management** (A, u, t, Space)
6. **Advanced Actions** (i, o, y, w)
7. **Settings Quick Access** (s, sp, sc, sl, sf)
8. **Cache Management** (Ctrl+r, Ctrl+Shift+r)

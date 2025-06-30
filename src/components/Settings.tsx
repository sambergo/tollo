import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface ChannelList {
  id: number;
  name: string;
  source: string; // url or file path
  is_default: boolean;
}

function Settings() {
  const [playerCommand, setPlayerCommand] = useState("");
  const [channelLists, setChannelLists] = useState<ChannelList[]>([]);
  const [newListName, setNewListName] = useState("");
  const [newListSource, setNewListSource] = useState("");
  const [defaultChannelList, setDefaultChannelList] = useState<number | null>(null);

  async function fetchPlayerCommand() {
    const fetchedCommand = await invoke<string>("get_player_command");
    setPlayerCommand(fetchedCommand);
  }

  async function fetchChannelLists() {
    const fetchedLists = await invoke<ChannelList[]>("get_channel_lists");
    setChannelLists(fetchedLists);
    const defaultList = fetchedLists.find((list) => list.is_default);
    if (defaultList) {
      setDefaultChannelList(defaultList.id);
    }
  }

  useEffect(() => {
    fetchPlayerCommand();
    fetchChannelLists();
  }, []);

  const handleSaveSettings = async () => {
    await invoke("set_player_command", { command: playerCommand });
    if (defaultChannelList !== null) {
      await invoke("set_default_channel_list", { id: defaultChannelList });
    }
  };

  const handleAddChannelList = async () => {
    if (newListName && newListSource) {
      await invoke("add_channel_list", { name: newListName, source: newListSource });
      setNewListName("");
      setNewListSource("");
      fetchChannelLists();
    }
  };

  const handleSetDefault = (id: number) => {
    setDefaultChannelList(id);
  };

  return (
    <div className="settings">
      <div>
        <label>Player Command</label>
        <input
          type="text"
          value={playerCommand}
          onChange={(e) => setPlayerCommand(e.target.value)}
        />
      </div>
      <hr />
      <div>
        <h3>Channel Lists</h3>
        <ul>
          {channelLists.map((list) => (
            <li key={list.id}>
              <span>{list.name} ({list.source})</span>
              <button onClick={() => handleSetDefault(list.id)}>
                {defaultChannelList === list.id ? "Default" : "Set as Default"}
              </button>
            </li>
          ))}
        </ul>
        <div>
          <input
            type="text"
            placeholder="List Name"
            value={newListName}
            onChange={(e) => setNewListName(e.target.value)}
          />
          <input
            type="text"
            placeholder="URL or File Path"
            value={newListSource}
            onChange={(e) => setNewListSource(e.target.value)}
          />
          <button onClick={handleAddChannelList}>Add List</button>
        </div>
      </div>
      <hr />
      <button onClick={handleSaveSettings}>Save Settings</button>
    </div>
  );
}

export default Settings;


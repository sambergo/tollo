"use strict";
(() => {
  const element = (id) => document.getElementById(id);
  const storage = {
    get: () => {
      try {
        return localStorage.getItem("tollo-remote-key") || "";
      } catch {
        return "";
      }
    },
    set: (key) => {
      try {
        key
          ? localStorage.setItem("tollo-remote-key", key)
          : localStorage.removeItem("tollo-remote-key");
      } catch {
        /* Connection works without browser storage. */
      }
    },
  };
  const fragmentKey = new URLSearchParams(location.hash.slice(1)).get("key");
  if (fragmentKey) history.replaceState(null, "", location.pathname);
  let key = fragmentKey || storage.get();
  let connected = false;
  let library = { lists: [], channels: [], favorites: [], history: [] };
  let view = "channels";
  let offset = 0;
  let listId = "";
  let playback = {
    state: "stopped",
    channel: null,
    paused: false,
    muted: false,
    volume: 100,
  };
  let libraryBusy = false;
  let statusBusy = false;
  let librarySignature = "";

  function error(message) {
    element("error").textContent = message || "";
    element("error").hidden = !message;
  }
  function disconnect() {
    connected = false;
    key = "";
    storage.set("");
    element("remote").hidden = true;
    element("login").hidden = false;
    element("disconnect").hidden = true;
    element("connection").textContent = "Not connected";
  }
  async function api(path, body) {
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), 30000);
    try {
      const response = await fetch(`/api/${path}`, {
        method: body ? "POST" : "GET",
        headers: {
          Authorization: `Bearer ${key}`,
          ...(body ? { "Content-Type": "application/json" } : {}),
        },
        ...(body ? { body: JSON.stringify(body) } : {}),
        signal: controller.signal,
      });
      const data = await response.json().catch(() => ({}));
      if (response.status === 401) disconnect();
      if (!response.ok)
        throw new Error(data.error || `Request failed (${response.status}).`);
      element("connection").textContent = "Connected";
      return data;
    } catch (failure) {
      if (failure instanceof TypeError || failure.name === "AbortError") {
        element("connection").textContent = "Connection lost";
        throw new Error(
          "Could not reach Tollo. Keep it open on the TV computer and check your network.",
        );
      }
      throw failure;
    } finally {
      clearTimeout(timer);
    }
  }
  function query() {
    const parameters = new URLSearchParams({ view, offset: String(offset) });
    if (listId) parameters.set("list_id", listId);
    if (element("search").value.trim())
      parameters.set("search", element("search").value.trim());
    if (element("group").value) parameters.set("group", element("group").value);
    return `?${parameters}`;
  }
  async function loadLibrary(force = false) {
    if (libraryBusy || !connected) return;
    libraryBusy = true;
    const requestedQuery = query();
    try {
      const data = await api(`library${requestedQuery}`);
      if (requestedQuery !== query() || !connected) return;
      const signature = JSON.stringify(data);
      if (signature !== librarySignature || force) {
        library = data;
        librarySignature = signature;
        const select = element("playlist");
        select.replaceChildren(new Option("Default playlist", ""));
        library.lists.forEach((list) =>
          select.add(new Option(list.name, String(list.id))),
        );
        if (listId && !library.lists.some((list) => String(list.id) === listId))
          listId = "";
        select.value = listId;
        const groupSelect = element("group");
        const oldGroup = groupSelect.value;
        groupSelect.replaceChildren(new Option("All groups", ""));
        data.groups.forEach((group) =>
          groupSelect.add(new Option(group, group)),
        );
        groupSelect.value = [...groupSelect.options].some(
          (option) => option.value === oldGroup,
        )
          ? oldGroup
          : "";
        if (offset && offset >= data.total) {
          offset = Math.max(0, Math.floor((data.total - 1) / 150) * 150);
        }
        render();
      }
    } catch (failure) {
      error(failure.message);
    } finally {
      libraryBusy = false;
      if (requestedQuery !== query()) void loadLibrary(true);
    }
  }
  function render() {
    const channels = library.channels;
    const favorites = new Set(library.favorites.map((channel) => channel.name));
    element("count").textContent = library.total
      ? `${library.total.toLocaleString()} channels · ${offset + 1}–${offset + channels.length}`
      : view === "favorites"
        ? "No favourites here yet. Use the star beside a channel to add one."
        : "No channels found. Try another search, group, or playlist.";
    const fragment = document.createDocumentFragment();
    channels.forEach((channel) => {
      const card = document.createElement("div");
      card.className = `channel${channel.name === playback.channel ? " current" : ""}`;
      const play = document.createElement("button");
      play.className = "play";
      play.title = `Play ${channel.name} on the TV`;
      const name = document.createElement("strong");
      name.textContent = channel.name;
      const detail = document.createElement("small");
      detail.textContent =
        [channel.group, channel.resolution].filter(Boolean).join(" · ") ||
        "Live channel";
      play.append(name, detail);
      play.onclick = () =>
        act({
          action: "play",
          channel_id: channel.id,
          list_id: listId ? Number(listId) : null,
        });
      const star = document.createElement("button");
      const favorite = favorites.has(channel.name);
      star.className = `star${favorite ? " active" : ""}`;
      star.textContent = favorite ? "★" : "☆";
      star.setAttribute(
        "aria-label",
        `${favorite ? "Remove" : "Add"} ${channel.name} ${favorite ? "from" : "to"} favourites`,
      );
      star.setAttribute("aria-pressed", String(favorite));
      star.onclick = async () => {
        star.disabled = true;
        error("");
        try {
          await api("favorite", {
            channel_id: channel.id,
            list_id: listId ? Number(listId) : null,
            enabled: !favorite,
          });
          await loadLibrary(true);
        } catch (failure) {
          error(failure.message);
          star.disabled = false;
        }
      };
      card.append(play, star);
      fragment.append(card);
    });
    element("channels").replaceChildren(fragment);
    element("more").hidden = offset + channels.length >= library.total;
    element("previous").hidden = offset === 0;
  }
  async function refreshStatus() {
    if (!connected || statusBusy) return;
    statusBusy = true;
    try {
      const status = await api("status");
      if (!connected) return;
      const changed = status.channel !== playback.channel;
      playback = status;
      element("playing-channel").textContent =
        status.channel || "Choose a channel";
      element("playback-state").textContent =
        {
          stopped: "Stopped",
          loading: "Loading…",
          playing: "Playing",
          paused: "Paused",
          failed: "Failed",
        }[status.state] || status.state;
      element("playback-error").textContent = status.error || "";
      element("playback-error").hidden = !status.error;
      element("pause").textContent = status.paused ? "Resume" : "Pause";
      element("mute").textContent = status.muted ? "Unmute" : "Mute";
      element("pause").disabled = !["playing", "paused"].includes(status.state);
      element("stop").disabled = !status.channel;
      element("mute").disabled = !status.channel;
      element("volume").disabled = !status.channel;
      if (document.activeElement !== element("volume")) {
        element("volume").value = String(
          Math.min(100, Math.round(status.volume)),
        );
        element("volume-value").textContent = `${Math.round(status.volume)}%`;
      }
      if (changed) render();
    } catch (failure) {
      error(failure.message);
    } finally {
      statusBusy = false;
    }
  }
  async function act(body) {
    error("");
    if (body.action === "play")
      element("playback-state").textContent = "Loading…";
    try {
      await api("control", body);
      await loadLibrary();
    } catch (failure) {
      error(failure.message);
    } finally {
      await refreshStatus();
    }
  }
  async function connect() {
    error("");
    try {
      await api("status");
      connected = true;
      storage.set(key);
      element("login").hidden = true;
      element("remote").hidden = false;
      element("disconnect").hidden = false;
      await loadLibrary(true);
      await refreshStatus();
    } catch (failure) {
      error(failure.message);
    }
  }
  element("connect-form").onsubmit = (event) => {
    event.preventDefault();
    key = element("key").value.trim();
    void connect();
  };
  element("disconnect").onclick = () => {
    disconnect();
    error("");
  };
  element("refresh").onclick = () => {
    error("");
    void loadLibrary(true);
    void refreshStatus();
  };
  element("playlist").onchange = () => {
    listId = element("playlist").value;
    offset = 0;
    void loadLibrary(true);
  };
  let searchTimer;
  element("search").oninput = () => {
    offset = 0;
    clearTimeout(searchTimer);
    searchTimer = setTimeout(() => {
      void loadLibrary(true);
    }, 150);
  };
  element("group").onchange = () => {
    offset = 0;
    void loadLibrary(true);
  };
  document.querySelectorAll("[data-tab]").forEach((button) => {
    button.onclick = () => {
      view = button.dataset.tab;
      offset = 0;
      document.querySelectorAll("[data-tab]").forEach((tab) => {
        tab.classList.toggle("selected", tab === button);
        tab.setAttribute("aria-pressed", String(tab === button));
      });
      void loadLibrary(true);
    };
  });
  element("more").onclick = () => {
    offset += 150;
    void loadLibrary(true);
  };
  element("previous").onclick = () => {
    offset = Math.max(0, offset - 150);
    void loadLibrary(true);
  };
  element("pause").onclick = () => act({ action: "pause" });
  element("stop").onclick = () => act({ action: "stop" });
  element("mute").onclick = () => act({ action: "mute" });
  element("volume").oninput = () => {
    element("volume-value").textContent = `${element("volume").value}%`;
  };
  element("volume").onchange = () =>
    act({ action: "volume", volume: Number(element("volume").value) });
  setInterval(() => {
    void refreshStatus();
  }, 1500);
  setInterval(() => {
    void loadLibrary();
  }, 10000);
  if (key) void connect();
})();

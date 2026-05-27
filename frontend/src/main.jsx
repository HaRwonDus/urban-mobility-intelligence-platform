import React, { useEffect, useMemo, useRef, useState } from 'react';
import { createRoot } from 'react-dom/client';
import { Activity, AlertTriangle, BusFront, Layers, MapPin, Route } from 'lucide-react';
import './styles.css';

const DISTRICT_LABELS = {
  Almalinsky: 'Almalinsky',
  Auezovsky: 'Auezovsky',
  Bostandyk: 'Bostandyk',
  Nauryzbay: 'Nauryzbay',
  Turksib: 'Turksib',
  Medeu: 'Medeu',
  Zhetysu: 'Zhetysu',
  Alatau: 'Alatau',
};

const fallbackDistricts = [
  { id: 'almalinsky', name: 'Almalinsky', score: 78, stop: 7, hub: 14, lat: 43.2489, lon: 76.9286 },
  { id: 'auezovsky', name: 'Auezovsky', score: 54, stop: 13, hub: 21, lat: 43.2327, lon: 76.8477 },
  { id: 'bostandyk', name: 'Bostandyk', score: 68, stop: 10, hub: 18, lat: 43.2034, lon: 76.9067 },
  { id: 'nauryzbay', name: 'Nauryzbay', score: 31, stop: 19, hub: 27, lat: 43.1972, lon: 76.7825 },
  { id: 'turksib', name: 'Turksib', score: 43, stop: 16, hub: 24, lat: 43.3335, lon: 76.987 },
  { id: 'medeu', name: 'Medeu', score: 62, stop: 12, hub: 20, lat: 43.2244, lon: 76.9958 },
  { id: 'zhetysu', name: 'Zhetysu', score: 47, stop: 15, hub: 23, lat: 43.2901, lon: 76.935 },
  { id: 'alatau', name: 'Alatau', score: 39, stop: 18, hub: 26, lat: 43.3006, lon: 76.8287 },
];

const fallbackRecommendations = [
  {
    area: 'Nauryzbay',
    problem: 'Long access time to metro or transfer hub',
    recommendation: 'Evaluate a transfer hub or trunk-route connection for this district.',
    confidence: 0.76,
  },
  {
    area: 'Bostandyk',
    problem: 'High POI density with weak stop access',
    recommendation: 'Add a new stop cluster or express feeder route near the strongest POI concentration.',
    confidence: 0.82,
  },
  {
    area: 'Zhetysu',
    problem: 'High POI density with weak stop access',
    recommendation: 'Add a new stop cluster or express feeder route near the strongest POI concentration.',
    confidence: 0.82,
  },
];

function scoreClass(score) {
  if (score < 40) return 'bad';
  if (score < 60) return 'weak';
  if (score < 80) return 'good';
  return 'great';
}

function normalizeDistrict(district, score) {
  return {
    id: district.id,
    name: DISTRICT_LABELS[district.name] || district.name,
    score: Math.round(score?.score ?? 50),
    stop: Math.round(score?.avg_time_to_stop_min ?? 15),
    hub: Math.round(score?.avg_time_to_hub_min ?? 22),
    poiDensity: Math.round(score?.poi_density ?? 0),
    connectivity: Math.round(score?.connectivity_score ?? 0),
    lat: district.lat,
    lon: district.lon,
  };
}

function loadMapglScript() {
  if (window.mapgl) return Promise.resolve(window.mapgl);

  return new Promise((resolve, reject) => {
    const existing = document.querySelector('script[data-mapgl]');
    if (existing) {
      existing.addEventListener('load', () => resolve(window.mapgl));
      existing.addEventListener('error', reject);
      return;
    }

    const script = document.createElement('script');
    script.src = 'https://mapgl.2gis.com/api/js/v1';
    script.async = true;
    script.defer = true;
    script.dataset.mapgl = 'true';
    script.onload = () => resolve(window.mapgl);
    script.onerror = reject;
    document.head.appendChild(script);
  });
}

function MapPanel({ districts, selected, onSelect }) {
  const mapNode = useRef(null);
  const mapRef = useRef(null);
  const markersRef = useRef([]);
  const [mapApi, setMapApi] = useState(null);
  const mapKey = import.meta.env.VITE_DGIS_MAPGL_KEY;

  useEffect(() => {
    if (!mapNode.current || !mapKey) return undefined;
    let cancelled = false;

    loadMapglScript()
      .then((mapgl) => {
        if (cancelled || mapRef.current) return;
        mapRef.current = new mapgl.Map(mapNode.current, {
          key: mapKey,
          center: [76.9286, 43.2489],
          zoom: 10.5,
          zoomControl: 'centerRight',
        });
        setMapApi(mapgl);
      })
      .catch(() => {
        if (mapNode.current) mapNode.current.dataset.error = 'MapGL failed to load';
      });

    return () => {
      cancelled = true;
      markersRef.current.forEach((marker) => marker.destroy?.());
      markersRef.current = [];
      mapRef.current?.destroy?.();
      mapRef.current = null;
      setMapApi(null);
    };
  }, [mapKey]);

  useEffect(() => {
    if (!mapRef.current || !mapApi) return;
    markersRef.current.forEach((marker) => marker.destroy?.());
    markersRef.current = districts.map((district) => {
      const html = document.createElement('button');
      html.className = `map-heat ${scoreClass(district.score)} ${
        selected?.id === district.id ? 'focus' : ''
      }`;
      html.textContent = String(district.score);
      html.title = district.name;
      html.addEventListener('click', () => onSelect(district));

      return new mapApi.HtmlMarker(mapRef.current, {
        coordinates: [district.lon, district.lat],
        html,
        anchor: [32, 32],
      });
    });
  }, [districts, selected?.id, onSelect, mapApi]);

  useEffect(() => {
    if (mapRef.current && selected) {
      mapRef.current.setCenter([selected.lon, selected.lat], { duration: 350 });
    }
  }, [selected]);

  return (
    <section className="map-panel">
      <div ref={mapNode} className="map-surface real-map">
        {!mapKey && <div className="map-error">MapGL key is missing</div>}
      </div>
      <div className="map-label">2GIS MapGL</div>
    </section>
  );
}

function App() {
  const apiUrl = import.meta.env.VITE_API_URL || 'http://127.0.0.1:8000';
  const [districts, setDistricts] = useState(fallbackDistricts);
  const [selected, setSelected] = useState(fallbackDistricts[0]);
  const [syncState, setSyncState] = useState({ status: 'idle', message: '' });
  const [recommendations, setRecommendations] = useState(fallbackRecommendations);

  const avgScore = useMemo(
    () => Math.round(districts.reduce((sum, item) => sum + item.score, 0) / districts.length),
    [districts]
  );

  async function loadDistricts() {
    const response = await fetch(`${apiUrl}/api/districts`);
    if (!response.ok) return;
    const rawDistricts = await response.json();
    const loaded = await Promise.all(
      rawDistricts.map(async (district) => {
        const scoreResponse = await fetch(`${apiUrl}/api/districts/${district.id}/score`);
        const score = scoreResponse.ok ? await scoreResponse.json() : null;
        return normalizeDistrict(district, score);
      })
    );
    if (loaded.length > 0) {
      setDistricts(loaded);
      setSelected((current) => loaded.find((item) => item.id === current?.id) || loaded[0]);
    }
  }

  async function loadRecommendations() {
    try {
      const response = await fetch(`${apiUrl}/api/recommendations`);
      if (!response.ok) return;
      const data = await response.json();
      if (Array.isArray(data) && data.length > 0) setRecommendations(data);
    } catch {
      setRecommendations(fallbackRecommendations);
    }
  }

  async function sync2gis() {
    setSyncState({ status: 'loading', message: 'Syncing 2GIS data...' });
    try {
      const response = await fetch(`${apiUrl}/api/sync/2gis`, { method: 'POST' });
      if (!response.ok) throw new Error(await response.text());
      const data = await response.json();
      setSyncState({
        status: 'ok',
        message: `${data.objects_loaded} objects loaded, ${data.districts_updated} districts updated`,
      });
      await Promise.all([loadDistricts(), loadRecommendations()]);
    } catch (error) {
      setSyncState({
        status: 'error',
        message: error instanceof Error ? error.message : 'Sync failed',
      });
    }
  }

  useEffect(() => {
    loadDistricts();
    loadRecommendations();
  }, []);

  return (
    <main className="app-shell">
      <aside className="sidebar">
        <div className="brand">
          <BusFront size={28} />
          <div>
            <h1>Urban Auditor</h1>
            <p>AI mobility analytics</p>
          </div>
        </div>

        <nav className="nav">
          <button className="active"><Layers size={18} /> Heatmap</button>
          <button><Route size={18} /> Routes</button>
          <button><Activity size={18} /> Forecast</button>
        </nav>

        <section className="metric-block">
          <span>Average score</span>
          <strong>{avgScore}</strong>
        </section>

        <section className="district-list">
          {districts.map((district) => (
            <button
              key={district.id}
              className={selected?.id === district.id ? 'selected' : ''}
              onClick={() => setSelected(district)}
            >
              <span>{district.name}</span>
              <b className={scoreClass(district.score)}>{district.score}</b>
            </button>
          ))}
        </section>
      </aside>

      <section className="workspace">
        <header className="topbar">
          <div>
            <h2>Transport Accessibility of Almaty</h2>
            <p>{selected?.name}: {selected?.stop} min to stop, {selected?.hub} min to hub</p>
          </div>
          <div className="sync-actions">
            <button className="primary" onClick={sync2gis} disabled={syncState.status === 'loading'}>
              <MapPin size={18} /> {syncState.status === 'loading' ? 'Syncing...' : 'Sync 2GIS'}
            </button>
            {syncState.message && <span className={syncState.status}>{syncState.message}</span>}
          </div>
        </header>

        <div className="content-grid">
          <MapPanel districts={districts} selected={selected} onSelect={setSelected} />

          <section className="inspector">
            <h3>{selected?.name}</h3>
            <div className="score-ring">
              <span>{selected?.score}</span>
              <small>accessibility</small>
            </div>
            <dl>
              <div><dt>To stop</dt><dd>{selected?.stop} min</dd></div>
              <div><dt>To hub</dt><dd>{selected?.hub} min</dd></div>
              <div><dt>POI density</dt><dd>{selected?.poiDensity}</dd></div>
              <div><dt>Status</dt><dd>{selected?.score < 60 ? 'needs upgrade' : 'stable'}</dd></div>
            </dl>
          </section>
        </div>

        <section className="recommendations">
          <h3><AlertTriangle size={18} /> AI recommendations</h3>
          <div className="recommendation-grid">
            {recommendations.map((item) => (
              <article key={`${item.id || item.area}-${item.confidence}-${item.problem}`}>
                <span>{item.area}</span>
                <h4>{item.problem}</h4>
                <p>{item.recommendation}</p>
                <b>{Math.round(item.confidence * 100)}%</b>
              </article>
            ))}
          </div>
        </section>
      </section>
    </main>
  );
}

createRoot(document.getElementById('root')).render(<App />);

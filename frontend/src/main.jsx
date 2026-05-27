import React, { useMemo, useState } from 'react';
import { createRoot } from 'react-dom/client';
import { Activity, AlertTriangle, BusFront, Layers, MapPin, Route } from 'lucide-react';
import './styles.css';

const districts = [
  { name: 'Алмалинский', score: 78, stop: 7, hub: 14, x: 38, y: 42 },
  { name: 'Ауэзовский', score: 54, stop: 13, hub: 21, x: 24, y: 52 },
  { name: 'Бостандыкский', score: 68, stop: 10, hub: 18, x: 44, y: 64 },
  { name: 'Наурызбайский', score: 31, stop: 19, hub: 27, x: 15, y: 68 },
  { name: 'Турксибский', score: 43, stop: 16, hub: 24, x: 69, y: 34 },
  { name: 'Медеуский', score: 62, stop: 12, hub: 20, x: 61, y: 58 },
];

const recommendations = [
  {
    area: 'Наурызбайский',
    problem: 'Высокое время доступа к магистральному транспорту',
    recommendation: 'Проверить экспресс-маршрут до БРТ-коридора',
    confidence: 0.82,
  },
  {
    area: 'Турксибский',
    problem: 'Слабая связность с больницами и вокзалом',
    recommendation: 'Добавить пересадочный хаб у плотного кластера остановок',
    confidence: 0.74,
  },
  {
    area: 'Ауэзовский',
    problem: 'Дублирование маршрутов на центральном коридоре',
    recommendation: 'Перераспределить часть рейсов в соседний underserved-сектор',
    confidence: 0.69,
  },
];

function scoreClass(score) {
  if (score < 40) return 'bad';
  if (score < 60) return 'weak';
  if (score < 80) return 'good';
  return 'great';
}

function App() {
  const [selected, setSelected] = useState(districts[0]);
  const avgScore = useMemo(
    () => Math.round(districts.reduce((sum, item) => sum + item.score, 0) / districts.length),
    []
  );

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
          <span>Средний score</span>
          <strong>{avgScore}</strong>
        </section>

        <section className="district-list">
          {districts.map((district) => (
            <button
              key={district.name}
              className={selected.name === district.name ? 'selected' : ''}
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
            <h2>Транспортная доступность Алматы</h2>
            <p>{selected.name}: {selected.stop} мин до остановки, {selected.hub} мин до хаба</p>
          </div>
          <button className="primary"><MapPin size={18} /> Sync 2GIS</button>
        </header>

        <div className="content-grid">
          <section className="map-panel">
            <div className="map-surface">
              {districts.map((district) => (
                <button
                  key={district.name}
                  className={`heat ${scoreClass(district.score)} ${selected.name === district.name ? 'focus' : ''}`}
                  style={{ left: `${district.x}%`, top: `${district.y}%` }}
                  onClick={() => setSelected(district)}
                  title={district.name}
                >
                  {district.score}
                </button>
              ))}
              <div className="route-line one" />
              <div className="route-line two" />
              <div className="map-label">2GIS MapGL layer</div>
            </div>
          </section>

          <section className="inspector">
            <h3>{selected.name}</h3>
            <div className="score-ring">
              <span>{selected.score}</span>
              <small>accessibility</small>
            </div>
            <dl>
              <div><dt>До остановки</dt><dd>{selected.stop} мин</dd></div>
              <div><dt>До хаба</dt><dd>{selected.hub} мин</dd></div>
              <div><dt>Статус</dt><dd>{selected.score < 60 ? 'нужно усиление' : 'стабильно'}</dd></div>
            </dl>
          </section>
        </div>

        <section className="recommendations">
          <h3><AlertTriangle size={18} /> AI-рекомендации</h3>
          <div className="recommendation-grid">
            {recommendations.map((item) => (
              <article key={`${item.area}-${item.confidence}`}>
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


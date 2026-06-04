import React, { useEffect, useMemo, useRef, useState } from 'react';
import { createRoot } from 'react-dom/client';
import {
  Activity,
  AlertTriangle,
  ArrowRight,
  BusFront,
  Building2,
  CircleDot,
  GitCompare,
  Gauge,
  Layers,
  MapPin,
  Plane,
  PlusCircle,
  Route,
  Shuffle,
  Sparkles,
  TrainFront,
  Wind,
} from 'lucide-react';
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

const fallbackRoutes = {
  gaps: [
    {
      id: 'nauryzbay-almalinsky',
      origin: 'Nauryzbay',
      destination: 'Almalinsky',
      current_time_min: 66,
      target_time_min: 35,
      problem: 'weak trunk connection',
      suggestion: 'express feeder route to metro/BRT hub',
      priority: 'high',
    },
    {
      id: 'alatau-bostandyk',
      origin: 'Alatau',
      destination: 'Bostandyk',
      current_time_min: 58,
      target_time_min: 37,
      problem: 'weak cross-city connection',
      suggestion: 'limited-stop connector through transfer hub',
      priority: 'medium',
    },
  ],
  suggestions: [
    {
      id: 'R-01',
      name: 'Nauryzbay to Almalinsky express',
      origin: 'Nauryzbay',
      via: ['Auezovsky'],
      destination: 'Almalinsky',
      route_type: 'express bus / BRT feeder',
      expected_impact: 18,
      confidence: 0.78,
    },
    {
      id: 'R-02',
      name: 'Alatau to Bostandyk connector',
      origin: 'Alatau',
      via: ['Zhetysu'],
      destination: 'Bostandyk',
      route_type: 'express bus',
      expected_impact: 14,
      confidence: 0.72,
    },
  ],
  duplicated_coverage: [
    {
      corridor: 'Auezovsky central corridor',
      problem: 'too many overlapping routes',
      action: 'redistribute part of trips to underserved sector',
      severity: 'medium',
    },
  ],
  comparison: [
    { district: 'Nauryzbay', current_score: 31, projected_score: 51, time_saved_min: 28, affected_poi: 19, priority: 'high' },
    { district: 'Alatau', current_score: 39, projected_score: 56, time_saved_min: 22, affected_poi: 18, priority: 'high' },
    { district: 'Auezovsky', current_score: 54, projected_score: 64, time_saved_min: 14, affected_poi: 13, priority: 'medium' },
  ],
};

const fallbackAirQuality = [
  {
    district: 'Almalinsky',
    lat: 43.2489,
    lon: 76.9286,
    aqi_us: 74,
    category: 'Moderate',
    main_pollutant: 'PM2.5',
    health_note: 'watch school and hospital corridors',
    source: 'MVP estimated layer; add IQAIR_API_KEY for live IQAir data',
  },
  {
    district: 'Auezovsky',
    lat: 43.2327,
    lon: 76.8477,
    aqi_us: 86,
    category: 'Moderate',
    main_pollutant: 'PM2.5',
    health_note: 'watch school and hospital corridors',
    source: 'MVP estimated layer; add IQAIR_API_KEY for live IQAir data',
  },
  {
    district: 'Nauryzbay',
    lat: 43.1972,
    lon: 76.7825,
    aqi_us: 105,
    category: 'Unhealthy for sensitive groups',
    main_pollutant: 'PM2.5',
    health_note: 'prioritize low-emission transit links',
    source: 'MVP estimated layer; add IQAIR_API_KEY for live IQAir data',
  },
  {
    district: 'Bostandyk',
    lat: 43.2034,
    lon: 76.9067,
    aqi_us: 79,
    category: 'Moderate',
    main_pollutant: 'PM2.5',
    health_note: 'watch school and hospital corridors',
    source: 'MVP estimated layer; add IQAIR_API_KEY for live IQAir data',
  },
  {
    district: 'Turksib',
    lat: 43.3335,
    lon: 76.987,
    aqi_us: 96,
    category: 'Moderate',
    main_pollutant: 'PM2.5',
    health_note: 'watch school and hospital corridors',
    source: 'MVP estimated layer; add IQAIR_API_KEY for live IQAir data',
  },
  {
    district: 'Medeu',
    lat: 43.2244,
    lon: 76.9958,
    aqi_us: 58,
    category: 'Moderate',
    main_pollutant: 'PM2.5',
    health_note: 'watch school and hospital corridors',
    source: 'MVP estimated layer; add IQAIR_API_KEY for live IQAir data',
  },
  {
    district: 'Zhetysu',
    lat: 43.2901,
    lon: 76.935,
    aqi_us: 91,
    category: 'Moderate',
    main_pollutant: 'PM2.5',
    health_note: 'watch school and hospital corridors',
    source: 'MVP estimated layer; add IQAIR_API_KEY for live IQAir data',
  },
  {
    district: 'Alatau',
    lat: 43.3006,
    lon: 76.8287,
    aqi_us: 112,
    category: 'Unhealthy for sensitive groups',
    main_pollutant: 'PM2.5',
    health_note: 'prioritize low-emission transit links',
    source: 'MVP estimated layer; add IQAIR_API_KEY for live IQAir data',
  },
];

const fallbackTraffic = [
  { id: 'traffic-nauryzbay', corridor: 'Nauryzbay main corridor', district: 'Nauryzbay', congestion_index: 0.72, average_speed_kmh: 25, delay_min: 13, source: 'estimated fallback' },
  { id: 'traffic-alatau', corridor: 'Alatau main corridor', district: 'Alatau', congestion_index: 0.68, average_speed_kmh: 27, delay_min: 12, source: 'estimated fallback' },
  { id: 'traffic-auezovsky', corridor: 'Auezovsky main corridor', district: 'Auezovsky', congestion_index: 0.54, average_speed_kmh: 31, delay_min: 10, source: 'estimated fallback' },
];

const fallbackVehicles = [
  { id: 'veh-01', route_id: 'R-01', route_name: 'Nauryzbay to Almalinsky express', transport_type: 'express bus / BRT feeder', lat: 43.221, lon: 76.842, occupancy: 71, delay_min: 4 },
  { id: 'veh-02', route_id: 'R-02', route_name: 'Alatau to Bostandyk connector', transport_type: 'express bus', lat: 43.266, lon: 76.866, occupancy: 65, delay_min: 5 },
  { id: 'veh-03', route_id: 'R-03', route_name: 'Turksib to Medeu express', transport_type: 'express bus', lat: 43.286, lon: 76.991, occupancy: 79, delay_min: 6 },
];

const fallbackRouteAnalysis = {
  route_name: 'Pilot public transport route',
  transport_type: 'bus',
  total_distance_km: 14.8,
  estimated_duration_min: 58,
  city_need: {
    score: 76,
    level: 'high',
    summary: 'Route is strongly justified for underserved districts',
    signals: ['Average accessibility score on corridor: 46', 'Average time to transfer hub: 23 min', 'Estimated covered population: 690000'],
  },
  duplication: {
    score: 38,
    level: 'low',
    summary: 'Low duplication, the line covers a distinct corridor',
    signals: ['Route overlap index: 0.38', 'Existing vehicles near corridor: 2', 'Districts crossed: Nauryzbay, Auezovsky, Almalinsky'],
  },
  overload_risk: {
    score: 61,
    level: 'medium',
    summary: 'Moderate overload risk during peaks',
    signals: ['Estimated demand: 1110 pax/hour', 'Planned capacity: 1020 pax/hour', 'Traffic pressure index: 0.55'],
  },
  recommendation: 'Approve for scenario modelling, then refine the corridor using live traffic and AVL data.',
  confidence: 0.74,
  data_sources: ['PostGIS district accessibility metrics', 'Traffic API layer', 'Public transport geolocation API layer'],
};

const fallbackHubs = [
  {
    id: 'airport',
    name: 'Almaty International Airport',
    hub_type: 'airport',
    lat: 43.3521,
    lon: 77.0405,
    avg_daily_arrivals: 11500,
    avg_daily_departures: 11600,
    avg_daily_flow: 23100,
    nearest_district: 'Turksib',
    access_time_min: 17,
    pressure_index: 82,
    recommendation: 'Add express airport transit and protect peak-hour bus priority.',
  },
  {
    id: 'almaty-2',
    name: 'Almaty-2 Railway Station',
    hub_type: 'station',
    lat: 43.2638,
    lon: 76.9455,
    avg_daily_arrivals: 9800,
    avg_daily_departures: 10100,
    avg_daily_flow: 19900,
    nearest_district: 'Almalinsky',
    access_time_min: 9,
    pressure_index: 65,
    recommendation: 'Monitor peak flows and improve first/last-mile coverage.',
  },
  {
    id: 'almaty-1',
    name: 'Almaty-1 Railway Station',
    hub_type: 'station',
    lat: 43.3417,
    lon: 76.9398,
    avg_daily_arrivals: 8200,
    avg_daily_departures: 7900,
    avg_daily_flow: 16100,
    nearest_district: 'Turksib',
    access_time_min: 14,
    pressure_index: 68,
    recommendation: 'Add feeder routes, park-and-ride capacity and dedicated transfer stops.',
  },
];

const fallbackCoverageGaps = [
  { id: 'nauryzbay', district: 'Nauryzbay', lat: 43.1972, lon: 76.7825, severity: 'high', isolation_score: 82, reason: '19 min to stop, 27 min to intermodal hub' },
  { id: 'alatau', district: 'Alatau', lat: 43.3006, lon: 76.8287, severity: 'high', isolation_score: 76, reason: '18 min to stop, 26 min to intermodal hub' },
  { id: 'turksib', district: 'Turksib', lat: 43.3335, lon: 76.987, severity: 'medium', isolation_score: 61, reason: '16 min to stop, 24 min to intermodal hub' },
];

const fallbackHubProposal = {
  name: 'Proposed station',
  hub_type: 'station',
  lat: 43.3006,
  lon: 76.8287,
  nearest_district: 'Alatau',
  underserved_score: 78,
  network_fit_score: 74,
  duplicate_pressure: 18,
  estimated_daily_flow: 13200,
  verdict: 'Strong candidate for a new intermodal hub scenario.',
  signals: ['Nearest district: Alatau', 'District accessibility score: 39', 'Time to existing hub: 26 min', 'Nearest same-type hub: 11.8 km'],
};

const DISTRICT_I18N = {
  Almalinsky: { ru: 'Алмалинский', kz: 'Алмалы' },
  Auezovsky: { ru: 'Ауэзовский', kz: 'Әуезов' },
  Bostandyk: { ru: 'Бостандыкский', kz: 'Бостандық' },
  Nauryzbay: { ru: 'Наурызбайский', kz: 'Наурызбай' },
  Turksib: { ru: 'Турксибский', kz: 'Түрксіб' },
  Medeu: { ru: 'Медеуский', kz: 'Медеу' },
  Zhetysu: { ru: 'Жетысуский', kz: 'Жетісу' },
  Alatau: { ru: 'Алатауский', kz: 'Алатау' },
};

const UI_TEXT = {
  ru: {
    subtitle: 'AI-аналитика мобильности',
    heatmap: 'Карта',
    routes: 'Маршруты',
    forecast: 'Сценарии',
    avgScore: 'Средний индекс',
    title: 'Транспортная доступность Алматы',
    stopHub: ({ name, stop, hub }) => `${name}: ${stop} мин до остановки, ${hub} мин до хаба`,
    syncing: 'Синхронизация...',
    sync: 'Обновить 2GIS',
    accessibility: 'Доступность',
    airQuality: 'Качество воздуха',
    coverageGaps: 'Отрезанные районы',
    mapMissing: 'Ключ MapGL не найден',
    accessibilityShort: 'доступность',
    toStop: 'До остановки',
    toHub: 'До хаба',
    poiDensity: 'Плотность POI',
    status: 'Статус',
    needsUpgrade: 'нужно улучшение',
    stable: 'стабильно',
    usAqi: 'US AQI',
    category: 'Категория',
    pollutant: 'Загрязнитель',
    healthNote: 'Рекомендация',
    source: 'Источник',
    estimated: 'оценка MVP',
    aiRecommendations: 'AI-рекомендации',
    routeGapDetector: 'Поиск слабых связей',
    current: 'Сейчас',
    target: 'Цель',
    simulateImprovement: 'Смоделировать улучшение',
    fromDistrict: 'Из района',
    toDistrict: 'В район',
    intervention: 'Мера',
    runSelectedPair: 'Запустить пару',
    routeBuilder: 'Конструктор маршрута ОТ',
    routePointMap: 'Карта точек маршрута',
    mapRoutePicker: 'Поставьте точку',
    routePoint: 'Точка',
    setPoint: 'Ставим',
    resetPoints: 'Сбросить точки',
    routeName: 'Название маршрута',
    viaDistrict: 'Через район',
    transportType: 'Тип ОТ',
    frequency: 'Интервал',
    vehicles: 'Машин на линии',
    analyzeRoute: 'Проанализировать маршрут',
    aiRouteVerdict: 'AI-оценка маршрута',
    cityNeed: 'Нужность городу',
    duplicationRisk: 'Дублирование',
    overloadRisk: 'Риск перегруза',
    trafficApi: 'API трафика',
    vehicleApi: 'Геопозиция ОТ',
    avgSpeed: 'средняя скорость',
    delay: 'задержка',
    occupancy: 'загрузка',
    duration: 'время',
    distance: 'длина',
    badConnections: 'Слабые связи',
    suggestion: 'Предложение',
    suggestedRoutes: 'Предложенные линии',
    confidence: 'уверенность',
    type: 'Тип',
    expectedImpact: 'Ожидаемый эффект',
    accessibilityPoints: 'пунктов доступности',
    duplicatedCoverage: 'Дублирование покрытия',
    simulationResult: 'Результат симуляции',
    saved: 'экономия',
    score: 'индекс',
    networkCompare: 'Текущая сеть vs предложенная сеть',
    district: 'Район',
    projected: 'После',
    poi: 'POI',
    priority: 'Приоритет',
    airLayer: 'Слой качества воздуха',
    intercityHubs: 'Аэропорт и ЖД хабы',
    arrivals: 'прибытие',
    departures: 'убытие',
    dailyFlow: 'пассажиров/день',
    hubPressure: 'нагрузка хаба',
    proposedHub: 'Предложение хаба',
    proposalType: 'Тип хаба',
    useSelectedPoint: 'Взять выбранную точку',
    analyzeHub: 'Оценить хаб',
    networkMode: 'Сценарий сети',
    existingNetwork: 'существующая сеть',
    greenfieldNetwork: 'проект с нуля',
    cutOffDistricts: 'Красные точки покрытия',
    isolation: 'изоляция',
    highestAqi: (name) => `${name} показывает самый высокий AQI на текущем слое карты.`,
    forecastTitle: 'Сценарная аналитика инфраструктуры',
    forecastLead: 'Forecast лучше делать как симулятор решений: что будет без изменений, после хабов, после усиления ОТ и после экологичных коридоров.',
    forecastNow: 'Что делать с Forecast',
    forecastNowText: 'Оставляем его не как гадание, а как сценарный модуль на основе маршрутов, доступности, воздуха и инфраструктурных дефицитов.',
    infraRecommendations: 'Предложения по инфраструктуре',
    scenarios: 'Сценарии улучшения',
    impactMatrix: 'Матрица эффекта',
    cost: 'Стоимость',
    impact: 'Эффект',
    difficulty: 'Сложность',
    scoreGain: 'Рост индекса',
    airGain: 'Воздух',
    medium: 'средний',
    high: 'высокий',
    low: 'низкий',
  },
  kz: {
    subtitle: 'AI мобильділік аналитикасы',
    heatmap: 'Карта',
    routes: 'Бағыттар',
    forecast: 'Сценарийлер',
    avgScore: 'Орташа индекс',
    title: 'Алматының көлік қолжетімділігі',
    stopHub: ({ name, stop, hub }) => `${name}: аялдамаға ${stop} мин, хабқа ${hub} мин`,
    syncing: 'Жаңартылуда...',
    sync: '2GIS жаңарту',
    accessibility: 'Қолжетімділік',
    airQuality: 'Ауа сапасы',
    coverageGaps: 'Оқшау аудандар',
    mapMissing: 'MapGL кілті табылмады',
    accessibilityShort: 'қолжетімділік',
    toStop: 'Аялдамаға',
    toHub: 'Хабқа',
    poiDensity: 'POI тығыздығы',
    status: 'Күйі',
    needsUpgrade: 'жақсарту қажет',
    stable: 'тұрақты',
    usAqi: 'US AQI',
    category: 'Санат',
    pollutant: 'Ластаушы',
    healthNote: 'Ұсыным',
    source: 'Дереккөз',
    estimated: 'MVP бағасы',
    aiRecommendations: 'AI ұсыныстары',
    routeGapDetector: 'Әлсіз байланысты іздеу',
    current: 'Қазір',
    target: 'Мақсат',
    simulateImprovement: 'Жақсартуды модельдеу',
    fromDistrict: 'Қай ауданнан',
    toDistrict: 'Қай ауданға',
    intervention: 'Шара',
    runSelectedPair: 'Жұпты іске қосу',
    routeBuilder: 'ОТ бағытын құрастыру',
    routePointMap: 'Бағыт нүктелерінің картасы',
    mapRoutePicker: 'Нүктені қойыңыз',
    routePoint: 'Нүкте',
    setPoint: 'Қою',
    resetPoints: 'Нүктелерді тазарту',
    routeName: 'Бағыт атауы',
    viaDistrict: 'Аралық аудан',
    transportType: 'ОТ түрі',
    frequency: 'Интервал',
    vehicles: 'Желідегі көлік',
    analyzeRoute: 'Бағытты талдау',
    aiRouteVerdict: 'AI бағыт бағасы',
    cityNeed: 'Қалаға қажеттілік',
    duplicationRisk: 'Қайталану',
    overloadRisk: 'Артық жүктеме қаупі',
    trafficApi: 'Трафик API',
    vehicleApi: 'ОТ геопозициясы',
    avgSpeed: 'орташа жылдамдық',
    delay: 'кешігу',
    occupancy: 'жүктеме',
    duration: 'уақыт',
    distance: 'ұзындығы',
    badConnections: 'Әлсіз байланыстар',
    suggestion: 'Ұсыныс',
    suggestedRoutes: 'Ұсынылған желілер',
    confidence: 'сенімділік',
    type: 'Түрі',
    expectedImpact: 'Күтілетін әсер',
    accessibilityPoints: 'қолжетімділік пункті',
    duplicatedCoverage: 'Қамтудың қайталануы',
    simulationResult: 'Симуляция нәтижесі',
    saved: 'үнем',
    score: 'индекс',
    networkCompare: 'Қазіргі желі vs ұсынылған желі',
    district: 'Аудан',
    projected: 'Кейін',
    poi: 'POI',
    priority: 'Басымдық',
    airLayer: 'Ауа сапасы қабаты',
    intercityHubs: 'Әуежай және ЖД хабтары',
    arrivals: 'келу',
    departures: 'кету',
    dailyFlow: 'жолаушы/күн',
    hubPressure: 'хаб жүктемесі',
    proposedHub: 'Хаб ұсынысы',
    proposalType: 'Хаб түрі',
    useSelectedPoint: 'Таңдалған нүктені алу',
    analyzeHub: 'Хабты бағалау',
    networkMode: 'Желі сценарийі',
    existingNetwork: 'қазіргі желі',
    greenfieldNetwork: 'нөлден жоба',
    cutOffDistricts: 'Қызыл қамту нүктелері',
    isolation: 'оқшаулау',
    highestAqi: (name) => `${name} ағымдағы карта қабатында ең жоғары AQI көрсетеді.`,
    forecastTitle: 'Инфрақұрылым сценарийлері',
    forecastLead: 'Forecast шешім симуляторы болуы керек: өзгеріссіз, хабтармен, ОТ күшейтуімен және экологиялық дәліздермен не өзгереді.',
    forecastNow: 'Forecast-пен не істейміз',
    forecastNowText: 'Оны болжау үшін емес, бағыттар, қолжетімділік, ауа сапасы және инфрақұрылым тапшылығы негізіндегі сценарий модулі ретінде қалдырамыз.',
    infraRecommendations: 'Инфрақұрылым ұсыныстары',
    scenarios: 'Жақсарту сценарийлері',
    impactMatrix: 'Әсер матрицасы',
    cost: 'Құны',
    impact: 'Әсер',
    difficulty: 'Күрделілік',
    scoreGain: 'Индекс өсімі',
    airGain: 'Ауа',
    medium: 'орташа',
    high: 'жоғары',
    low: 'төмен',
  },
};

const PHRASE_I18N = {
  'weak trunk connection': {
    ru: 'слабая магистральная связь',
    kz: 'магистральдық байланыс әлсіз',
  },
  'weak cross-city connection': {
    ru: 'слабая межрайонная связь',
    kz: 'ауданаралық байланыс әлсіз',
  },
  'express feeder route to metro/BRT hub': {
    ru: 'экспресс-фидер к метро/BRT-хабу',
    kz: 'метро/BRT хабына экспресс-фидер',
  },
  'limited-stop connector through transfer hub': {
    ru: 'скоростной коннектор через пересадочный хаб',
    kz: 'ауысып отыру хабы арқылы жылдам байланыс',
  },
  'express bus / BRT feeder': {
    ru: 'экспресс-автобус / BRT-фидер',
    kz: 'экспресс автобус / BRT-фидер',
  },
  'express bus': {
    ru: 'экспресс-автобус',
    kz: 'экспресс автобус',
  },
  'too many overlapping routes': {
    ru: 'слишком много пересекающихся маршрутов',
    kz: 'қайталанатын бағыттар көп',
  },
  'redistribute part of trips to underserved sector': {
    ru: 'перераспределить часть рейсов в недообслуженный сектор',
    kz: 'рейстердің бір бөлігін қамтылмаған секторға бөлу',
  },
  'Moderate': {
    ru: 'Умеренное',
    kz: 'Орташа',
  },
  'Unhealthy for sensitive groups': {
    ru: 'Вредно для чувствительных групп',
    kz: 'Сезімтал топтарға зиян',
  },
  'watch school and hospital corridors': {
    ru: 'контролировать коридоры школ и больниц',
    kz: 'мектеп пен аурухана дәліздерін бақылау',
  },
  'prioritize low-emission transit links': {
    ru: 'приоритизировать низкоэмиссионные транзитные связи',
    kz: 'төмен эмиссиялы транзит байланыстарын күшейту',
  },
  'Long access time to metro or transfer hub': {
    ru: 'Долгий доступ к метро или пересадочному хабу',
    kz: 'Метроға немесе ауысу хабына жету уақыты ұзақ',
  },
  'High POI density with weak stop access': {
    ru: 'Высокая плотность POI при слабом доступе к остановкам',
    kz: 'POI тығыз, бірақ аялдамаға қолжетімділік әлсіз',
  },
  'Evaluate a transfer hub or trunk-route connection for this district.': {
    ru: 'Проверить пересадочный хаб или магистральную связь для района.',
    kz: 'Аудан үшін ауысу хабын немесе магистральдық байланысты тексеру.',
  },
  'Add a new stop cluster or express feeder route near the strongest POI concentration.': {
    ru: 'Добавить кластер остановок или экспресс-фидер около сильной концентрации POI.',
    kz: 'POI шоғырланған жерде аялдама кластерін немесе экспресс-фидер қосу.',
  },
};

const INTERVENTIONS = [
  { value: 'express feeder', ru: 'экспресс-фидер', kz: 'экспресс-фидер' },
  { value: 'BRT corridor', ru: 'BRT-коридор', kz: 'BRT дәлізі' },
  { value: 'new hub', ru: 'новый хаб', kz: 'жаңа хаб' },
  { value: 'add 10 stops', ru: 'добавить 10 остановок', kz: '10 аялдама қосу' },
];

function tt(lang, key, params) {
  const value = UI_TEXT[lang][key] || UI_TEXT.ru[key] || key;
  return typeof value === 'function' ? value(params || {}) : value;
}

function districtLabel(name, lang) {
  return DISTRICT_I18N[name]?.[lang] || name;
}

function phrase(value, lang) {
  return PHRASE_I18N[value]?.[lang] || value;
}

function priorityLabel(value, lang) {
  return UI_TEXT[lang][value] || value;
}

function scoreClass(score) {
  if (score < 40) return 'bad';
  if (score < 60) return 'weak';
  if (score < 80) return 'good';
  return 'great';
}

function airClass(aqi) {
  if (aqi <= 50) return 'air-good';
  if (aqi <= 100) return 'air-moderate';
  if (aqi <= 150) return 'air-sensitive';
  return 'air-unhealthy';
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

function createRouteStopsFromDistricts(districts) {
  const find = (name, fallbackIndex) => districts.find((district) => district.name === name) || districts[fallbackIndex] || fallbackDistricts[fallbackIndex];
  const points = [
    ['A', find('Nauryzbay', 3)],
    ['B', find('Auezovsky', 1)],
    ['C', find('Almalinsky', 0)],
  ];

  return points.map(([label, district]) => ({
    label,
    name: `${label}: ${district.name}`,
    lat: district.lat,
    lon: district.lon,
  }));
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

function MapPanel({
  districts,
  airQuality,
  coverageGaps,
  selected,
  mapMode,
  lang,
  onSelect,
  routeStops = [],
  activeRoutePointIndex = 0,
  onRoutePointSelect,
  routePickMode = false,
}) {
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
    const points = mapMode === 'air' ? airQuality : mapMode === 'coverage' ? coverageGaps : districts;
    markersRef.current = points.map((point) => {
      const district = mapMode === 'air'
        ? districts.find((item) => item.name === point.district) || point
        : mapMode === 'coverage'
          ? districts.find((item) => item.name === point.district) || point
        : point;
      const html = document.createElement('button');
      const value = mapMode === 'air' ? point.aqi_us : mapMode === 'coverage' ? point.isolation_score : district.score;
      const markerClass = mapMode === 'air'
        ? airClass(point.aqi_us)
        : mapMode === 'coverage'
          ? 'coverage-bad'
          : scoreClass(district.score);
      html.className = `map-heat ${markerClass} ${
        selected?.id === district.id ? 'focus' : ''
      }`;
      html.textContent = String(value);
      html.title = mapMode === 'air'
        ? `${districtLabel(point.district, lang)} AQI ${point.aqi_us}`
        : mapMode === 'coverage'
          ? `${districtLabel(point.district, lang)} ${tt(lang, 'isolation')} ${point.isolation_score}`
        : districtLabel(district.name, lang);
      html.addEventListener('click', () => onSelect(district));

      return new mapApi.HtmlMarker(mapRef.current, {
        coordinates: [point.lon, point.lat],
        html,
        anchor: [32, 32],
      });
    });

    const routeMarkers = routeStops.map((stop, index) => {
      const html = document.createElement('button');
      html.className = `route-point-marker ${index === activeRoutePointIndex ? 'active' : ''}`;
      html.textContent = stop.label;
      html.title = `${stop.label}: ${stop.name}`;
      html.addEventListener('click', (event) => {
        event.stopPropagation();
        onRoutePointSelect?.({ lat: stop.lat, lon: stop.lon }, index);
      });

      return new mapApi.HtmlMarker(mapRef.current, {
        coordinates: [stop.lon, stop.lat],
        html,
        anchor: [22, 22],
      });
    });

    markersRef.current = [...markersRef.current, ...routeMarkers];
  }, [districts, airQuality, coverageGaps, selected?.id, mapMode, onSelect, mapApi, routeStops, activeRoutePointIndex, onRoutePointSelect]);

  useEffect(() => {
    if (!mapRef.current || !routePickMode || !onRoutePointSelect) return undefined;
    const handler = (event) => {
      const raw = event?.lngLat || event?.coordinates || event?.targetData?.coordinates;
      const lon = Array.isArray(raw) ? raw[0] : raw?.lng ?? raw?.lon;
      const lat = Array.isArray(raw) ? raw[1] : raw?.lat;
      if (Number.isFinite(lat) && Number.isFinite(lon)) {
        onRoutePointSelect({ lat, lon }, activeRoutePointIndex);
      }
    };

    mapRef.current.on?.('click', handler);
    return () => mapRef.current?.off?.('click', handler);
  }, [routePickMode, activeRoutePointIndex, onRoutePointSelect, mapApi]);

  useEffect(() => {
    if (mapRef.current && selected) {
      mapRef.current.setCenter([selected.lon, selected.lat], { duration: 350 });
    }
  }, [selected]);

  return (
    <section className="map-panel">
      <div ref={mapNode} className="map-surface real-map">
        {!mapKey && <div className="map-error">{tt(lang, 'mapMissing')}</div>}
      </div>
      <div className="map-label">{routePickMode ? `${tt(lang, 'mapRoutePicker')} ${routeStops[activeRoutePointIndex]?.label || 'A'}` : '2GIS MapGL'}</div>
    </section>
  );
}

function RoutesView({
  routes,
  districts,
  simulation,
  routeDraft,
  setRouteDraft,
  onSimulate,
  isSimulating,
  analysis,
  onAnalyze,
  isAnalyzing,
  traffic,
  vehicles,
  activeRoutePointIndex,
  setActiveRoutePointIndex,
  onRoutePointSelect,
  onResetRoutePoints,
  lang,
}) {
  const topGap = routes.gaps?.[0];
  const districtNames = districts.map((district) => district.name);
  const topTraffic = [...traffic].sort((a, b) => b.congestion_index - a.congestion_index).slice(0, 3);
  const activeVehicles = vehicles.slice(0, 3);

  return (
    <div className="routes-layout">
      <section className="route-hero">
        <div>
          <span className="eyebrow">{tt(lang, 'routeGapDetector')}</span>
          <h2>{districtLabel(topGap?.origin || 'Nauryzbay', lang)} <ArrowRight size={22} /> {districtLabel(topGap?.destination || 'Almalinsky', lang)}</h2>
          <p>{phrase(topGap?.problem || 'weak trunk connection', lang)}</p>
        </div>
        <div className="route-time">
          <span>{tt(lang, 'current')}</span>
          <strong>{topGap?.current_time_min || 66} min</strong>
          <small>{tt(lang, 'target')} {topGap?.target_time_min || 35} min</small>
        </div>
        <button className="primary" onClick={() => onSimulate(topGap)} disabled={isSimulating || !topGap}>
          <Sparkles size={18} /> {isSimulating ? tt(lang, 'syncing') : tt(lang, 'simulateImprovement')}
        </button>
      </section>

      <section className="route-selector">
        <div>
          <label htmlFor="route-origin">{tt(lang, 'fromDistrict')}</label>
          <select
            id="route-origin"
            value={routeDraft.origin}
            onChange={(event) => setRouteDraft((current) => ({ ...current, origin: event.target.value }))}
          >
            {districtNames.map((name) => <option key={name} value={name}>{districtLabel(name, lang)}</option>)}
          </select>
        </div>
        <ArrowRight size={20} />
        <div>
          <label htmlFor="route-destination">{tt(lang, 'toDistrict')}</label>
          <select
            id="route-destination"
            value={routeDraft.destination}
            onChange={(event) => setRouteDraft((current) => ({ ...current, destination: event.target.value }))}
          >
            {districtNames.map((name) => <option key={name} value={name}>{districtLabel(name, lang)}</option>)}
          </select>
        </div>
        <div>
          <label htmlFor="route-intervention">{tt(lang, 'intervention')}</label>
          <select
            id="route-intervention"
            value={routeDraft.intervention}
            onChange={(event) => setRouteDraft((current) => ({ ...current, intervention: event.target.value }))}
          >
            {INTERVENTIONS.map((item) => (
              <option key={item.value} value={item.value}>{item[lang]}</option>
            ))}
          </select>
        </div>
        <button className="primary" onClick={() => onSimulate(routeDraft)} disabled={isSimulating}>
          <Sparkles size={18} /> {tt(lang, 'runSelectedPair')}
        </button>
      </section>

      <section className="route-builder-panel">
        <div className="builder-header">
          <div>
            <span className="eyebrow">{tt(lang, 'routeBuilder')}</span>
            <h3>{routeDraft.name}</h3>
          </div>
          <button className="primary" onClick={() => onAnalyze(routeDraft)} disabled={isAnalyzing}>
            <Sparkles size={18} /> {isAnalyzing ? tt(lang, 'syncing') : tt(lang, 'analyzeRoute')}
          </button>
        </div>

        <div className="builder-grid">
          <label>
            <span>{tt(lang, 'routeName')}</span>
            <input
              value={routeDraft.name}
              onChange={(event) => setRouteDraft((current) => ({ ...current, name: event.target.value }))}
            />
          </label>
          <label>
            <span>{tt(lang, 'viaDistrict')}</span>
            <select
              value={routeDraft.via}
              onChange={(event) => setRouteDraft((current) => ({ ...current, via: event.target.value }))}
            >
              {districtNames.map((name) => <option key={name} value={name}>{districtLabel(name, lang)}</option>)}
            </select>
          </label>
          <label>
            <span>{tt(lang, 'transportType')}</span>
            <select
              value={routeDraft.transport_type}
              onChange={(event) => setRouteDraft((current) => ({ ...current, transport_type: event.target.value }))}
            >
              <option value="bus">Bus</option>
              <option value="brt">BRT</option>
              <option value="trolleybus">Trolleybus</option>
              <option value="tram">Tram</option>
            </select>
          </label>
          <label>
            <span>{tt(lang, 'frequency')}</span>
            <input
              type="number"
              min="3"
              max="45"
              value={routeDraft.frequency_min}
              onChange={(event) => setRouteDraft((current) => ({ ...current, frequency_min: Number(event.target.value) }))}
            />
          </label>
          <label>
            <span>{tt(lang, 'vehicles')}</span>
            <input
              type="number"
              min="1"
              max="80"
              value={routeDraft.planned_vehicles}
              onChange={(event) => setRouteDraft((current) => ({ ...current, planned_vehicles: Number(event.target.value) }))}
            />
          </label>
          <label>
            <span>{tt(lang, 'networkMode')}</span>
            <select
              value={routeDraft.greenfield ? 'greenfield' : 'existing'}
              onChange={(event) => setRouteDraft((current) => ({ ...current, greenfield: event.target.value === 'greenfield' }))}
            >
              <option value="existing">{tt(lang, 'existingNetwork')}</option>
              <option value="greenfield">{tt(lang, 'greenfieldNetwork')}</option>
            </select>
          </label>
        </div>
      </section>

      <section className="route-point-workbench">
        <div className="route-panel route-map-panel">
          <div className="builder-header">
            <div>
              <span className="eyebrow">{tt(lang, 'routePointMap')}</span>
              <h3>{tt(lang, 'setPoint')} {routeDraft.stops[activeRoutePointIndex]?.label}</h3>
            </div>
            <button className="secondary" onClick={onResetRoutePoints}>
              <MapPin size={17} /> {tt(lang, 'resetPoints')}
            </button>
          </div>
          <MapPanel
            districts={districts}
            airQuality={[]}
            coverageGaps={[]}
            selected={null}
            mapMode="accessibility"
            lang={lang}
            onSelect={(district) => onRoutePointSelect({ lat: district.lat, lon: district.lon }, activeRoutePointIndex)}
            routeStops={routeDraft.stops}
            activeRoutePointIndex={activeRoutePointIndex}
            onRoutePointSelect={onRoutePointSelect}
            routePickMode
          />
        </div>

        <div className="route-panel route-stop-panel">
          <h3><MapPin size={18} /> {tt(lang, 'routePoint')}</h3>
          <div className="route-stop-list">
            {routeDraft.stops.map((stop, index) => (
              <button
                key={stop.label}
                className={index === activeRoutePointIndex ? 'active' : ''}
                onClick={() => setActiveRoutePointIndex(index)}
              >
                <strong>{stop.label}</strong>
                <span>{stop.name}</span>
                <small>{stop.lat.toFixed(4)}, {stop.lon.toFixed(4)}</small>
              </button>
            ))}
          </div>
        </div>
      </section>

      <section className="analysis-grid">
        <div className="route-panel analysis-panel">
          <h3><Sparkles size={18} /> {tt(lang, 'aiRouteVerdict')}</h3>
          <div className="analysis-summary">
            <strong>{analysis.route_name}</strong>
            <span>{tt(lang, 'distance')}: {analysis.total_distance_km} km</span>
            <span>{tt(lang, 'duration')}: {analysis.estimated_duration_min} min</span>
            <p>{analysis.recommendation}</p>
          </div>
          <div className="criteria-grid">
            <AnalysisScoreCard title={tt(lang, 'cityNeed')} criterion={analysis.city_need} />
            <AnalysisScoreCard title={tt(lang, 'duplicationRisk')} criterion={analysis.duplication} />
            <AnalysisScoreCard title={tt(lang, 'overloadRisk')} criterion={analysis.overload_risk} />
          </div>
        </div>

        <div className="route-panel live-data-panel">
          <h3><Gauge size={18} /> {tt(lang, 'trafficApi')}</h3>
          {topTraffic.map((item) => (
            <div key={item.id} className="live-row">
              <div>
                <b>{districtLabel(item.district, lang)}</b>
                <span>{item.corridor}</span>
              </div>
              <strong>{Math.round(item.congestion_index * 100)}%</strong>
              <small>{item.average_speed_kmh} km/h, {item.delay_min} min {tt(lang, 'delay')}</small>
            </div>
          ))}
        </div>

        <div className="route-panel live-data-panel">
          <h3><BusFront size={18} /> {tt(lang, 'vehicleApi')}</h3>
          {activeVehicles.map((item) => (
            <div key={item.id} className="live-row">
              <div>
                <b>{item.route_id}</b>
                <span>{item.route_name}</span>
              </div>
              <strong>{item.occupancy}%</strong>
              <small>{tt(lang, 'occupancy')}, {item.delay_min} min {tt(lang, 'delay')}</small>
            </div>
          ))}
        </div>
      </section>

      <section className="routes-grid">
        <div className="route-panel">
          <h3><AlertTriangle size={18} /> {tt(lang, 'badConnections')}</h3>
          <div className="route-card-list">
            {routes.gaps.map((gap) => (
              <article key={gap.id} className="route-card">
                <div>
                  <h4>{districtLabel(gap.origin, lang)} <ArrowRight size={15} /> {districtLabel(gap.destination, lang)}</h4>
                  <span className={`priority ${gap.priority}`}>{priorityLabel(gap.priority, lang)}</span>
                </div>
                <dl>
                  <div><dt>{tt(lang, 'current')}</dt><dd>{gap.current_time_min} min</dd></div>
                  <div><dt>{tt(lang, 'target')}</dt><dd>{gap.target_time_min} min</dd></div>
                  <div><dt>{tt(lang, 'suggestion')}</dt><dd>{phrase(gap.suggestion, lang)}</dd></div>
                </dl>
              </article>
            ))}
          </div>
        </div>

        <div className="route-panel">
          <h3><Route size={18} /> {tt(lang, 'suggestedRoutes')}</h3>
          <div className="route-card-list">
            {routes.suggestions.map((route) => (
              <article key={route.id} className="route-card">
                <div>
                  <h4>{route.id}</h4>
                  <span>{Math.round(route.confidence * 100)}% {tt(lang, 'confidence')}</span>
                </div>
                <p>{districtLabel(route.origin, lang)} <ArrowRight size={14} /> {route.via.map((name) => districtLabel(name, lang)).join(' / ')} <ArrowRight size={14} /> {districtLabel(route.destination, lang)}</p>
                <dl>
                  <div><dt>{tt(lang, 'type')}</dt><dd>{phrase(route.route_type, lang)}</dd></div>
                  <div><dt>{tt(lang, 'expectedImpact')}</dt><dd>+{route.expected_impact} {tt(lang, 'accessibilityPoints')}</dd></div>
                </dl>
              </article>
            ))}
          </div>
        </div>
      </section>

      <section className="routes-grid lower">
        <div className="route-panel">
          <h3><Shuffle size={18} /> {tt(lang, 'duplicatedCoverage')}</h3>
          <div className="route-card-list">
            {routes.duplicated_coverage.map((item) => (
              <article key={item.corridor} className="route-card compact">
                <div>
                  <h4>{item.corridor.replace(/^([A-Za-z]+)/, (match) => districtLabel(match, lang))}</h4>
                  <span className={`priority ${item.severity}`}>{priorityLabel(item.severity, lang)}</span>
                </div>
                <p>{phrase(item.problem, lang)}</p>
                <b>{phrase(item.action, lang)}</b>
              </article>
            ))}
          </div>
        </div>

        <div className="route-panel simulate-panel">
          <h3><Sparkles size={18} /> {tt(lang, 'simulationResult')}</h3>
          <div className="simulate-result">
            <span>{districtLabel(simulation.origin, lang)} <ArrowRight size={15} /> {districtLabel(simulation.destination, lang)}</span>
            <strong>{simulation.current_time_min} min -&gt; {simulation.projected_time_min} min</strong>
            <p>{phrase(simulation.suggested_route, lang)}</p>
            <b>{simulation.time_saved_min} min {tt(lang, 'saved')}, +{simulation.projected_score_gain} {tt(lang, 'score')}</b>
          </div>
        </div>
      </section>

      <section className="comparison-panel">
        <h3><GitCompare size={18} /> {tt(lang, 'networkCompare')}</h3>
        <div className="comparison-table">
          <div className="table-head">
            <span>{tt(lang, 'district')}</span>
            <span>{tt(lang, 'current')}</span>
            <span>{tt(lang, 'projected')}</span>
            <span>{tt(lang, 'saved')}</span>
            <span>{tt(lang, 'poi')}</span>
            <span>{tt(lang, 'priority')}</span>
          </div>
          {routes.comparison.map((row) => (
            <div key={row.district} className="table-row">
              <span>{districtLabel(row.district, lang)}</span>
              <span>{row.current_score}</span>
              <span>{row.projected_score}</span>
              <span>{row.time_saved_min} min</span>
              <span>{row.affected_poi}</span>
              <span className={`priority ${row.priority}`}>{priorityLabel(row.priority, lang)}</span>
            </div>
          ))}
        </div>
      </section>
    </div>
  );
}

function AnalysisScoreCard({ title, criterion }) {
  return (
    <article className={`criterion-card ${criterion.level}`}>
      <div>
        <span>{title}</span>
        <strong>{criterion.score}</strong>
      </div>
      <p>{criterion.summary}</p>
      <ul>
        {criterion.signals.slice(0, 3).map((signal) => <li key={signal}>{signal}</li>)}
      </ul>
    </article>
  );
}

function AirQualityPanel({ airQuality, lang }) {
  const sorted = [...airQuality].sort((a, b) => b.aqi_us - a.aqi_us);
  const worst = sorted[0];

  return (
    <section className="air-panel">
      <div>
        <h3><Wind size={18} /> {tt(lang, 'airLayer')}</h3>
        <p>{tt(lang, 'highestAqi', { name: districtLabel(worst?.district || 'Almaty', lang) })}</p>
      </div>
      <div className="air-grid">
        {sorted.slice(0, 4).map((item) => (
          <article key={item.district} className="air-card">
            <div>
              <span>{districtLabel(item.district, lang)}</span>
              <b className={airClass(item.aqi_us)}>{item.aqi_us}</b>
            </div>
            <h4>{phrase(item.category, lang)}</h4>
            <p>{item.main_pollutant}: {phrase(item.health_note, lang)}</p>
          </article>
        ))}
      </div>
      <small>{worst?.source || 'IQAir AirVisual API ready'}</small>
    </section>
  );
}

function HubsPanel({ hubs, lang }) {
  return (
    <section className="hub-panel">
      <div>
        <h3><TrainFront size={18} /> {tt(lang, 'intercityHubs')}</h3>
        <p>{tt(lang, 'dailyFlow')} и {tt(lang, 'hubPressure')} для ключевых внешних ворот города.</p>
      </div>
      <div className="hub-grid">
        {hubs.slice(0, 4).map((hub) => (
          <article key={hub.id} className="hub-card">
            <div>
              {hub.hub_type === 'airport' ? <Plane size={19} /> : <TrainFront size={19} />}
              <span>{hub.hub_type}</span>
            </div>
            <h4>{hub.name}</h4>
            <dl>
              <div><dt>{tt(lang, 'arrivals')}</dt><dd>{hub.avg_daily_arrivals}</dd></div>
              <div><dt>{tt(lang, 'departures')}</dt><dd>{hub.avg_daily_departures}</dd></div>
              <div><dt>{tt(lang, 'hubPressure')}</dt><dd>{hub.pressure_index}</dd></div>
            </dl>
            <p>{hub.recommendation}</p>
          </article>
        ))}
      </div>
    </section>
  );
}

function HubProposalPanel({ proposal, setProposal, analysis, selected, onUseSelected, onAnalyze, isAnalyzing, lang }) {
  return (
    <section className="hub-proposal-panel">
      <div className="builder-header">
        <div>
          <span className="eyebrow">{tt(lang, 'proposedHub')}</span>
          <h3>{analysis.name}</h3>
        </div>
        <button className="primary" onClick={onAnalyze} disabled={isAnalyzing}>
          <PlusCircle size={18} /> {isAnalyzing ? tt(lang, 'syncing') : tt(lang, 'analyzeHub')}
        </button>
      </div>
      <div className="proposal-grid">
        <label>
          <span>{tt(lang, 'routeName')}</span>
          <input value={proposal.name} onChange={(event) => setProposal((current) => ({ ...current, name: event.target.value }))} />
        </label>
        <label>
          <span>{tt(lang, 'proposalType')}</span>
          <select value={proposal.hub_type} onChange={(event) => setProposal((current) => ({ ...current, hub_type: event.target.value }))}>
            <option value="station">ЖД вокзал</option>
            <option value="bus_station">Автовокзал</option>
          </select>
        </label>
        <label>
          <span>Lat</span>
          <input type="number" step="0.0001" value={proposal.lat} onChange={(event) => setProposal((current) => ({ ...current, lat: Number(event.target.value) }))} />
        </label>
        <label>
          <span>Lon</span>
          <input type="number" step="0.0001" value={proposal.lon} onChange={(event) => setProposal((current) => ({ ...current, lon: Number(event.target.value) }))} />
        </label>
        <button className="secondary" onClick={onUseSelected}>
          <CircleDot size={17} /> {tt(lang, 'useSelectedPoint')}
        </button>
      </div>
      <div className="proposal-result">
        <strong>{analysis.network_fit_score}</strong>
        <div>
          <b>{districtLabel(analysis.nearest_district, lang)}</b>
          <p>{analysis.verdict}</p>
          <span>{tt(lang, 'isolation')}: {analysis.underserved_score}, {tt(lang, 'duplicatedCoverage')}: {analysis.duplicate_pressure}, flow: {analysis.estimated_daily_flow}</span>
        </div>
      </div>
    </section>
  );
}

function CoverageGapPanel({ coverageGaps, lang }) {
  return (
    <section className="coverage-panel">
      <h3><AlertTriangle size={18} /> {tt(lang, 'cutOffDistricts')}</h3>
      <div className="coverage-grid">
        {coverageGaps.map((gap) => (
          <article key={gap.id} className="coverage-card">
            <span className={`priority ${gap.severity}`}>{priorityLabel(gap.severity, lang)}</span>
            <h4>{districtLabel(gap.district, lang)}</h4>
            <strong>{gap.isolation_score}</strong>
            <p>{gap.reason}</p>
          </article>
        ))}
      </div>
    </section>
  );
}

function buildInfrastructureRecommendations(districts, airQuality, routes, lang) {
  const airByDistrict = new Map(airQuality.map((item) => [item.district, item]));
  const routePriority = new Map(
    routes.gaps.map((gap) => [gap.origin, { destination: gap.destination, saved: gap.current_time_min - gap.target_time_min }])
  );

  return districts
    .map((district) => {
      const air = airByDistrict.get(district.name);
      const routeGap = routePriority.get(district.name);
      const needHub = district.hub >= 23 || district.score < 42;
      const needOt = district.stop >= 15 || routeGap;
      const needGreen = (air?.aqi_us || 0) >= 95;
      const score = (needHub ? 34 : 0) + (needOt ? 32 : 0) + (needGreen ? 24 : 0) + Math.max(0, 60 - district.score) / 3;

      return {
        district: district.name,
        title: needHub
          ? { ru: 'Пересадочный хаб + фидерные маршруты', kz: 'Ауысу хабы + фидер бағыттары' }
          : needOt
            ? { ru: 'Усиление остановочной сети и ОТ', kz: 'Аялдама желісі мен ОТ күшейту' }
            : { ru: 'Точечная модернизация городской среды', kz: 'Қалалық ортаны нүктелік жаңарту' },
        action: needGreen
          ? {
              ru: 'Запустить низкоэмиссионный коридор, приоритет ОТ на перекрестках и озеленение вдоль перегруженных улиц.',
              kz: 'Төмен эмиссиялы дәліз, қиылыстарда ОТ басымдығы және жүктелген көшелер бойында көгалдандыру енгізу.',
            }
          : {
              ru: 'Добавить удобные пересадки, павильоны, пешеходные подходы и расписание под магистральные линии.',
              kz: 'Ыңғайлы ауысулар, павильондар, жаяу қолжетімділік және магистральдық желілерге сай кесте қосу.',
            },
        kpi: routeGap
          ? `-${routeGap.saved} min ${districtLabel(routeGap.destination, lang)}`
          : `+${Math.max(8, Math.round((70 - district.score) / 2))} ${tt(lang, 'score')}`,
        priority: score > 72 ? 'high' : score > 46 ? 'medium' : 'low',
      };
    })
    .sort((a, b) => ['high', 'medium', 'low'].indexOf(a.priority) - ['high', 'medium', 'low'].indexOf(b.priority))
    .slice(0, 5);
}

function ForecastView({ districts, airQuality, routes, lang }) {
  const recommendations = buildInfrastructureRecommendations(districts, airQuality, routes, lang);
  const avgAqi = Math.round(airQuality.reduce((sum, item) => sum + item.aqi_us, 0) / airQuality.length);
  const routeGain = routes.comparison.reduce((sum, item) => sum + Math.max(0, item.projected_score - item.current_score), 0);
  const scenarios = [
    {
      id: 'A',
      title: { ru: 'Добавить 10 остановок', kz: '10 аялдама қосу' },
      cost: 'low',
      impact: 'medium',
      difficulty: 'low',
      scoreGain: '+8',
      airGain: avgAqi > 90 ? '-3 AQI' : '-1 AQI',
    },
    {
      id: 'B',
      title: { ru: '2 пересадочных хаба', kz: '2 ауысу хабы' },
      cost: 'medium',
      impact: 'high',
      difficulty: 'medium',
      scoreGain: `+${Math.max(14, Math.round(routeGain / 3))}`,
      airGain: '-4 AQI',
    },
    {
      id: 'C',
      title: { ru: 'Экспресс к метро/BRT', kz: 'Метро/BRT-ке экспресс' },
      cost: 'medium',
      impact: 'high',
      difficulty: 'medium',
      scoreGain: '+22',
      airGain: '-6 AQI',
    },
    {
      id: 'D',
      title: { ru: 'BRT + зеленый коридор', kz: 'BRT + жасыл дәліз' },
      cost: 'high',
      impact: 'high',
      difficulty: 'high',
      scoreGain: '+29',
      airGain: '-10 AQI',
    },
  ];

  return (
    <div className="forecast-layout">
      <section className="forecast-hero">
        <div>
          <span className="eyebrow">Forecast MVP</span>
          <h2>{tt(lang, 'forecastTitle')}</h2>
          <p>{tt(lang, 'forecastLead')}</p>
        </div>
        <div className="forecast-note">
          <h3>{tt(lang, 'forecastNow')}</h3>
          <p>{tt(lang, 'forecastNowText')}</p>
        </div>
      </section>

      <section className="forecast-grid">
        <div className="route-panel">
          <h3><Building2 size={18} /> {tt(lang, 'infraRecommendations')}</h3>
          <div className="route-card-list">
            {recommendations.map((item) => (
              <article key={item.district} className="route-card compact">
                <div>
                  <h4>{districtLabel(item.district, lang)}</h4>
                  <span className={`priority ${item.priority}`}>{priorityLabel(item.priority, lang)}</span>
                </div>
                <b>{item.title[lang]}</b>
                <p>{item.action[lang]}</p>
                <span>{item.kpi}</span>
              </article>
            ))}
          </div>
        </div>

        <div className="route-panel">
          <h3><Gauge size={18} /> {tt(lang, 'scenarios')}</h3>
          <div className="scenario-list">
            {scenarios.map((scenario) => (
              <article key={scenario.id} className="scenario-card">
                <div>
                  <strong>{scenario.id}</strong>
                  <h4>{scenario.title[lang]}</h4>
                </div>
                <dl>
                  <div><dt>{tt(lang, 'cost')}</dt><dd>{priorityLabel(scenario.cost, lang)}</dd></div>
                  <div><dt>{tt(lang, 'impact')}</dt><dd>{priorityLabel(scenario.impact, lang)}</dd></div>
                  <div><dt>{tt(lang, 'difficulty')}</dt><dd>{priorityLabel(scenario.difficulty, lang)}</dd></div>
                  <div><dt>{tt(lang, 'scoreGain')}</dt><dd>{scenario.scoreGain}</dd></div>
                  <div><dt>{tt(lang, 'airGain')}</dt><dd>{scenario.airGain}</dd></div>
                </dl>
              </article>
            ))}
          </div>
        </div>
      </section>
    </div>
  );
}

function App() {
  const apiUrl = import.meta.env.VITE_API_URL || 'http://127.0.0.1:8000';
  const [lang, setLang] = useState('ru');
  const [districts, setDistricts] = useState(fallbackDistricts);
  const [selected, setSelected] = useState(fallbackDistricts[0]);
  const [syncState, setSyncState] = useState({ status: 'idle', message: '' });
  const [recommendations, setRecommendations] = useState(fallbackRecommendations);
  const [activeTab, setActiveTab] = useState('heatmap');
  const [mapMode, setMapMode] = useState('accessibility');
  const [routes, setRoutes] = useState(fallbackRoutes);
  const [airQuality, setAirQuality] = useState(fallbackAirQuality);
  const [hubs, setHubs] = useState(fallbackHubs);
  const [coverageGaps, setCoverageGaps] = useState(fallbackCoverageGaps);
  const [routeDraft, setRouteDraft] = useState({
    name: 'Pilot public transport route',
    origin: 'Nauryzbay',
    via: 'Auezovsky',
    destination: 'Almalinsky',
    intervention: 'express feeder',
    transport_type: 'bus',
    frequency_min: 10,
    planned_vehicles: 8,
    greenfield: false,
    stops: createRouteStopsFromDistricts(fallbackDistricts),
  });
  const [hubProposal, setHubProposal] = useState({
    name: 'Proposed Alatau rail hub',
    hub_type: 'station',
    lat: 43.3006,
    lon: 76.8287,
    daily_capacity: 14000,
    greenfield: false,
  });
  const [hubProposalAnalysis, setHubProposalAnalysis] = useState(fallbackHubProposal);
  const [traffic, setTraffic] = useState(fallbackTraffic);
  const [vehicles, setVehicles] = useState(fallbackVehicles);
  const [routeAnalysis, setRouteAnalysis] = useState(fallbackRouteAnalysis);
  const [routeSimulation, setRouteSimulation] = useState({
    origin: 'Nauryzbay',
    destination: 'Almalinsky',
    current_time_min: 66,
    suggested_route: 'express feeder',
    projected_time_min: 38,
    time_saved_min: 28,
    projected_score_gain: 20,
    confidence: 0.78,
  });
  const [isSimulating, setIsSimulating] = useState(false);
  const [isAnalyzingRoute, setIsAnalyzingRoute] = useState(false);
  const [isAnalyzingHub, setIsAnalyzingHub] = useState(false);
  const [activeRoutePointIndex, setActiveRoutePointIndex] = useState(0);

  const avgScore = useMemo(
    () => Math.round(districts.reduce((sum, item) => sum + item.score, 0) / districts.length),
    [districts]
  );
  const selectedAir = useMemo(
    () => airQuality.find((item) => item.district === selected?.name) || airQuality[0],
    [airQuality, selected?.name]
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
      setRouteDraft((current) => ({
        ...current,
        stops: current.stops?.some((stop) => stop.name.startsWith('A: Nauryzbay'))
          ? createRouteStopsFromDistricts(loaded)
          : current.stops,
      }));
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

  async function loadRoutes() {
    try {
      const response = await fetch(`${apiUrl}/api/routes`);
      if (!response.ok) return;
      const data = await response.json();
      if (data?.gaps?.length) setRoutes(data);
    } catch {
      setRoutes(fallbackRoutes);
    }
  }

  async function loadAirQuality() {
    try {
      const response = await fetch(`${apiUrl}/api/air-quality`);
      if (!response.ok) return;
      const data = await response.json();
      if (Array.isArray(data) && data.length > 0) setAirQuality(data);
    } catch {
      setAirQuality(fallbackAirQuality);
    }
  }

  async function loadMobilityLayers() {
    try {
      const [trafficResponse, vehiclesResponse] = await Promise.all([
        fetch(`${apiUrl}/api/mobility/traffic`),
        fetch(`${apiUrl}/api/mobility/public-transport/locations`),
      ]);
      if (trafficResponse.ok) {
        const trafficData = await trafficResponse.json();
        if (Array.isArray(trafficData) && trafficData.length > 0) setTraffic(trafficData);
      }
      if (vehiclesResponse.ok) {
        const vehicleData = await vehiclesResponse.json();
        if (Array.isArray(vehicleData) && vehicleData.length > 0) setVehicles(vehicleData);
      }
    } catch {
      setTraffic(fallbackTraffic);
      setVehicles(fallbackVehicles);
    }
  }

  async function loadHubsAndCoverage() {
    try {
      const [hubsResponse, gapsResponse] = await Promise.all([
        fetch(`${apiUrl}/api/hubs`),
        fetch(`${apiUrl}/api/coverage/gaps`),
      ]);
      if (hubsResponse.ok) {
        const data = await hubsResponse.json();
        if (Array.isArray(data) && data.length > 0) setHubs(data);
      }
      if (gapsResponse.ok) {
        const data = await gapsResponse.json();
        if (Array.isArray(data) && data.length > 0) setCoverageGaps(data);
      }
    } catch {
      setHubs(fallbackHubs);
      setCoverageGaps(fallbackCoverageGaps);
    }
  }

  function buildRouteStops(draft) {
    if (draft.stops?.filter((stop) => Number.isFinite(stop.lat) && Number.isFinite(stop.lon)).length >= 2) {
      return draft.stops
        .filter((stop) => Number.isFinite(stop.lat) && Number.isFinite(stop.lon))
        .map((stop) => ({
          name: stop.name,
          lat: stop.lat,
          lon: stop.lon,
        }));
    }

    const byName = new Map(districts.map((district) => [district.name, district]));
    return [draft.origin, draft.via, draft.destination]
      .filter((name, index, items) => name && items.indexOf(name) === index)
      .map((name) => byName.get(name))
      .filter(Boolean)
      .map((district) => ({
        name: districtLabel(district.name, lang),
        lat: district.lat,
        lon: district.lon,
      }));
  }

  function setRoutePoint(point, pointIndex = activeRoutePointIndex) {
    setRouteDraft((current) => {
      const currentStops = current.stops?.length ? current.stops : createRouteStopsFromDistricts(districts);
      const nextStops = currentStops.map((stop, index) => {
        if (index !== pointIndex) return stop;
        const nearestDistrict = districts
          .map((district) => ({
            district,
            distance: Math.hypot(district.lat - point.lat, district.lon - point.lon),
          }))
          .sort((a, b) => a.distance - b.distance)[0]?.district;
        return {
          ...stop,
          name: `${stop.label}: ${nearestDistrict ? districtLabel(nearestDistrict.name, lang) : tt(lang, 'routePoint')}`,
          lat: Number(point.lat.toFixed(6)),
          lon: Number(point.lon.toFixed(6)),
        };
      });

      return { ...current, stops: nextStops };
    });
    setActiveRoutePointIndex((current) => Math.min(pointIndex + 1, 2));
  }

  function resetRoutePoints() {
    setRouteDraft((current) => ({
      ...current,
      stops: createRouteStopsFromDistricts(districts),
    }));
    setActiveRoutePointIndex(0);
  }

  async function analyzeRoute(draft) {
    const stops = buildRouteStops(draft);
    if (stops.length < 2) return;
    setIsAnalyzingRoute(true);
    try {
      const response = await fetch(`${apiUrl}/api/routes/analyze`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          name: draft.name,
          transport_type: draft.transport_type,
          stops,
          frequency_min: draft.frequency_min,
          planned_vehicles: draft.planned_vehicles,
          greenfield: draft.greenfield,
        }),
      });
      if (!response.ok) throw new Error('Route analysis failed');
      setRouteAnalysis(await response.json());
    } catch {
      setRouteAnalysis(fallbackRouteAnalysis);
    } finally {
      setIsAnalyzingRoute(false);
    }
  }

  async function analyzeHubProposal() {
    setIsAnalyzingHub(true);
    try {
      const response = await fetch(`${apiUrl}/api/hubs/proposals`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(hubProposal),
      });
      if (!response.ok) throw new Error('Hub analysis failed');
      setHubProposalAnalysis(await response.json());
    } catch {
      setHubProposalAnalysis(fallbackHubProposal);
    } finally {
      setIsAnalyzingHub(false);
    }
  }

  function useSelectedForHubProposal() {
    if (!selected) return;
    setHubProposal((current) => ({
      ...current,
      lat: Number(selected.lat.toFixed(4)),
      lon: Number(selected.lon.toFixed(4)),
      name: `${districtLabel(selected.name, lang)} ${current.hub_type === 'station' ? 'rail hub' : 'bus hub'}`,
    }));
    setMapMode('coverage');
  }

  async function simulateRoute(gap = routes.gaps[0]) {
    if (!gap) return;
    const origin = gap.origin;
    const destination = gap.destination;
    const intervention = gap.intervention || gap.suggestion || 'express feeder';
    setIsSimulating(true);
    try {
      const response = await fetch(`${apiUrl}/api/routes/simulate`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          origin,
          destination,
          intervention,
        }),
      });
      if (!response.ok) throw new Error('Simulation failed');
      setRouteSimulation(await response.json());
    } catch {
      setRouteSimulation({
        origin,
        destination,
        current_time_min: gap.current_time_min || 66,
        suggested_route: intervention,
        projected_time_min: Math.max((gap.target_time_min || 35) + 3, (gap.current_time_min || 66) - 28),
        time_saved_min: Math.min(28, (gap.current_time_min || 66) - (gap.target_time_min || 35)),
        projected_score_gain: 20,
        confidence: 0.78,
      });
    } finally {
      setIsSimulating(false);
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
    loadRoutes();
    loadAirQuality();
    loadMobilityLayers();
    loadHubsAndCoverage();
  }, []);

  return (
    <main className="app-shell">
      <aside className="sidebar">
        <div className="brand">
          <BusFront size={28} />
          <div>
            <h1>Urban Auditor</h1>
            <p>{tt(lang, 'subtitle')}</p>
          </div>
        </div>

        <div className="language-switcher" aria-label="Language">
          <button className={lang === 'ru' ? 'active' : ''} onClick={() => setLang('ru')}>RU</button>
          <button className={lang === 'kz' ? 'active' : ''} onClick={() => setLang('kz')}>KZ</button>
        </div>

        <nav className="nav">
          <button className={activeTab === 'heatmap' ? 'active' : ''} onClick={() => setActiveTab('heatmap')}><Layers size={18} /> {tt(lang, 'heatmap')}</button>
          <button className={activeTab === 'routes' ? 'active' : ''} onClick={() => setActiveTab('routes')}><Route size={18} /> {tt(lang, 'routes')}</button>
          <button className={activeTab === 'forecast' ? 'active' : ''} onClick={() => setActiveTab('forecast')}><Activity size={18} /> {tt(lang, 'forecast')}</button>
        </nav>

        <section className="metric-block">
          <span>{tt(lang, 'avgScore')}</span>
          <strong>{avgScore}</strong>
        </section>

        <section className="district-list">
          {districts.map((district) => (
            <button
              key={district.id}
              className={selected?.id === district.id ? 'selected' : ''}
              onClick={() => setSelected(district)}
            >
              <span>{districtLabel(district.name, lang)}</span>
              <b className={scoreClass(district.score)}>{district.score}</b>
            </button>
          ))}
        </section>
      </aside>

      <section className="workspace">
        <header className="topbar">
          <div>
            <h2>{tt(lang, 'title')}</h2>
            <p>{tt(lang, 'stopHub', { name: districtLabel(selected?.name, lang), stop: selected?.stop, hub: selected?.hub })}</p>
          </div>
          <div className="sync-actions">
            <button className="primary" onClick={sync2gis} disabled={syncState.status === 'loading'}>
              <MapPin size={18} /> {syncState.status === 'loading' ? tt(lang, 'syncing') : tt(lang, 'sync')}
            </button>
            {syncState.message && <span className={syncState.status}>{syncState.message}</span>}
          </div>
        </header>

        {activeTab === 'heatmap' && (
          <>
            <section className="layer-switcher">
              <button className={mapMode === 'accessibility' ? 'active' : ''} onClick={() => setMapMode('accessibility')}>
                <Layers size={17} /> {tt(lang, 'accessibility')}
              </button>
              <button className={mapMode === 'air' ? 'active' : ''} onClick={() => setMapMode('air')}>
                <Wind size={17} /> {tt(lang, 'airQuality')}
              </button>
              <button className={mapMode === 'coverage' ? 'active' : ''} onClick={() => setMapMode('coverage')}>
                <AlertTriangle size={17} /> {tt(lang, 'coverageGaps')}
              </button>
            </section>

            <div className="content-grid">
              <MapPanel
                districts={districts}
                airQuality={airQuality}
                coverageGaps={coverageGaps}
                selected={selected}
                mapMode={mapMode}
                lang={lang}
                onSelect={setSelected}
              />

              <section className="inspector">
                <h3>{districtLabel(selected?.name, lang)}</h3>
                {mapMode === 'air' ? (
                  <>
                    <div className={`aqi-ring ${airClass(selectedAir?.aqi_us || 0)}`}>
                      <span>{selectedAir?.aqi_us}</span>
                      <small>{tt(lang, 'usAqi')}</small>
                    </div>
                    <dl>
                      <div><dt>{tt(lang, 'category')}</dt><dd>{phrase(selectedAir?.category, lang)}</dd></div>
                      <div><dt>{tt(lang, 'pollutant')}</dt><dd>{selectedAir?.main_pollutant}</dd></div>
                      <div><dt>{tt(lang, 'healthNote')}</dt><dd>{phrase(selectedAir?.health_note, lang)}</dd></div>
                      <div><dt>{tt(lang, 'source')}</dt><dd>{selectedAir?.source?.includes('IQAir') ? 'IQAir' : tt(lang, 'estimated')}</dd></div>
                    </dl>
                  </>
                ) : (
                  <>
                    <div className="score-ring">
                      <span>{selected?.score}</span>
                      <small>{tt(lang, 'accessibilityShort')}</small>
                    </div>
                    <dl>
                      <div><dt>{tt(lang, 'toStop')}</dt><dd>{selected?.stop} min</dd></div>
                      <div><dt>{tt(lang, 'toHub')}</dt><dd>{selected?.hub} min</dd></div>
                      <div><dt>{tt(lang, 'poiDensity')}</dt><dd>{selected?.poiDensity}</dd></div>
                      <div><dt>{tt(lang, 'status')}</dt><dd>{selected?.score < 60 ? tt(lang, 'needsUpgrade') : tt(lang, 'stable')}</dd></div>
                    </dl>
                  </>
                )}
              </section>
            </div>

            {mapMode === 'air' && <AirQualityPanel airQuality={airQuality} lang={lang} />}
            {mapMode === 'coverage' && <CoverageGapPanel coverageGaps={coverageGaps} lang={lang} />}

            <HubsPanel hubs={hubs} lang={lang} />

            <HubProposalPanel
              proposal={hubProposal}
              setProposal={setHubProposal}
              analysis={hubProposalAnalysis}
              selected={selected}
              onUseSelected={useSelectedForHubProposal}
              onAnalyze={analyzeHubProposal}
              isAnalyzing={isAnalyzingHub}
              lang={lang}
            />

            <section className="recommendations">
              <h3><AlertTriangle size={18} /> {tt(lang, 'aiRecommendations')}</h3>
              <div className="recommendation-grid">
                {recommendations.map((item) => (
                  <article key={`${item.id || item.area}-${item.confidence}-${item.problem}`}>
                    <span>{districtLabel(item.area, lang)}</span>
                    <h4>{phrase(item.problem, lang)}</h4>
                    <p>{phrase(item.recommendation, lang)}</p>
                    <b>{Math.round(item.confidence * 100)}%</b>
                  </article>
                ))}
              </div>
            </section>
          </>
        )}

        {activeTab === 'routes' && (
          <RoutesView
            routes={routes}
            districts={districts}
            simulation={routeSimulation}
            routeDraft={routeDraft}
            setRouteDraft={setRouteDraft}
            onSimulate={simulateRoute}
            isSimulating={isSimulating}
            analysis={routeAnalysis}
            onAnalyze={analyzeRoute}
            isAnalyzing={isAnalyzingRoute}
            traffic={traffic}
            vehicles={vehicles}
            activeRoutePointIndex={activeRoutePointIndex}
            setActiveRoutePointIndex={setActiveRoutePointIndex}
            onRoutePointSelect={setRoutePoint}
            onResetRoutePoints={resetRoutePoints}
            lang={lang}
          />
        )}

        {activeTab === 'forecast' && (
          <ForecastView districts={districts} airQuality={airQuality} routes={routes} lang={lang} />
        )}
      </section>
    </main>
  );
}

createRoot(document.getElementById('root')).render(<App />);

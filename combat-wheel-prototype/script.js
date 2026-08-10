// State variables
const state = {
    selectedRay: null,      // 0..7 or 'center'
    thrustDirection: null,  // 0..7 (if selectedRay === 'center')
    thrustManner: 'active', // 'active' (Доведенный укол, точка ●) | 'interception' (Встречный / Набежал сам, крестик ┼)
    opposition: null,       // 'center' (взял) | 'edge' (не взял) | null
    edge: null              // 'left' | 'right' (для ударов) | 'center' (для уколов)
};

// Wheel Geometry Constants
const CX = 300;
const CY = 300;
const R_INNER = 55;
const R_OUTER = 245;

const RAY_ANGLES = [0, 45, 90, 135, 180, 225, 270, 315];
const RAY_NAMES = [
    "Верх (Ober)", "Верх-Право", "Право (Mittel)", "Низ-Право",
    "Низ (Unter)", "Низ-Лево", "Лево (Mittel)", "Верх-Лево"
];

// HEMA Liechtenauer / Meyer German Longsword Database
const TECHNIQUES = {
    // CUTS (Рубящие удары - Голубая палитра)
    '0_center_left': {
        title: "Шильхау (Schielhau)",
        desc: "Мастерский 'косящий' удар короткой стороной со скрещиванием рук. Перебивает прямые вертикальные удары и уколы сверху."
    },
    '0_center_right': {
        title: "Шайтельхау (Scheitelhau)",
        desc: "Прямой вертикальный мастерский удар длинным лезвием в темя соперника со взятием оппозиции и перекрытием центра."
    },
    '0_edge_left': {
        title: "Штурцхау (Sturzhau)",
        desc: "Падающий сверху хлесткий удар короткой стороной лезвия без захвата защиты."
    },
    '0_edge_right': {
        title: "Герадер Оберхау (Gerader Oberhau)",
        desc: "Прямой рубящий удар сверху вниз длинным лезвием."
    },

    '1_center_left': {
        title: "Крумпхау (Krumphau)",
        desc: "Мастерский 'кривой' удар короткой стороной с шагом в сторону. Бьет по кистям и перебивает позицию Окс."
    },
    '1_center_right': {
        title: "Цорнхау (Zornhau)",
        desc: "Мощный диагональный 'удар гнева' справа-налево с перекрытием атаки соперника и выходом на укол."
    },
    '1_edge_left': {
        title: "Курцер Оберхау (Kurzer Oberhau)",
        desc: "Быстрый кистевой щелчок короткой стороной сверху-справа."
    },
    '1_edge_right': {
        title: "Рехтер Оберхау (Rechter Oberhau)",
        desc: "Классический диагональный рубящий удар сверху-справа длинной стороной."
    },

    '2_center_left': {
        title: "Цверххау (Zwerchhau)",
        desc: "Горизонтальный мастерский удар короткой стороной с поднятым над головой эфесом. Прикрывает голову и разит в висок."
    },
    '2_center_right': {
        title: "Цверххау Длинной (Zwerchhau)",
        desc: "Горизонтальный перекрывающий удар длинной стороной в верхний или средний сектор."
    },
    '2_edge_left': {
        title: "Прелльхау (Prellhau)",
        desc: "Быстрый хлесткий горизонтальный щелчок короткой стороной справа."
    },
    '2_edge_right': {
        title: "Рехтер Миттельхау (Rechter Mittelhau)",
        desc: "Горизонтальный боковой удар по корпусу или ребрам справа длинным лезвием."
    },

    '3_center_left': {
        title: "Вексельхау (Wechselhau)",
        desc: "Подрезающий удар короткой стороной снизу-справа с оппозицией и выходом в стойку."
    },
    '3_center_right': {
        title: "Унтерхау со связыванием",
        desc: "Взлетающий удар снизу-справа длинным лезвием с отводом меча соперника."
    },
    '3_edge_left': {
        title: "Курцер Унтерхау (Kurzer Unterhau)",
        desc: "Короткий подрез снизу-справа короткой стороной."
    },
    '3_edge_right': {
        title: "Рехтер Унтерхау (Rechter Unterhau)",
        desc: "Диагональный восходящий удар снизу-справа длинным лезвием."
    },

    '4_center_left': {
        title: "Шильхау Снизу (Schielhau von unten)",
        desc: "Восходящий срез короткой стороной со скрещиванием рук и контролем гарды."
    },
    '4_center_right': {
        title: "Герадер Унтерхау со связыванием",
        desc: "Вертикальный снизу вверх удар длинной стороной со взятием оппозиции."
    },
    '4_edge_left': {
        title: "Шнеллер (Schneller)",
        desc: "Быстрый кистевой подрез строго снизу короткой стороной."
    },
    '4_edge_right': {
        title: "Герадер Унтерхау (Gerader Unterhau)",
        desc: "Прямой восходящий удар снизу вверх длинной стороной."
    },

    '5_center_left': {
        title: "Флюгельхау (Flügelhau)",
        desc: "Подрезающий удар короткой стороной снизу-слева со скрещением рук в оппозиции."
    },
    '5_center_right': {
        title: "Линкер Унтерхау с защитой",
        desc: "Восходящий подрез снизу-слева длинным лезвием с прикрытием корпуса."
    },
    '5_edge_left': {
        title: "Курцер Унтерхау Линкс",
        desc: "Быстрый кистевой подрез короткой стороной снизу-слева."
    },
    '5_edge_right': {
        title: "Линкер Унтерхау (Linker Unterhau)",
        desc: "Диагональный восходящий удар снизу-слева длинным лезвием."
    },

    '6_center_left': {
        title: "Линкер Цверххау (Linker Zwerchhau)",
        desc: "Поперечный мастерский удар слева короткой стороной над гардой с оппозицией."
    },
    '6_center_right': {
        title: "Линкер Цверххау Длинной",
        desc: "Поперечный удар слева длинным лезвием со связыванием оружия соперника."
    },
    '6_edge_left': {
        title: "Линкер Прелльхау (Linker Prellhau)",
        desc: "Боковой щелчок короткой стороной слева по голове или виску."
    },
    '6_edge_right': {
        title: "Линкер Миттельхау (Linker Mittelhau)",
        desc: "Горизонтальный рубящий удар длинным лезвием слева направо."
    },

    '7_center_left': {
        title: "Крумпхау Линкс (Krumphau Links)",
        desc: "Кривой мастерский удар короткой стороной сверху-слева по кистям соперника."
    },
    '7_center_right': {
        title: "Линкер Цорнхау (Zornhau Links)",
        desc: "Диагональный мастерский удар сверху-слева с наложением сильной части клинка."
    },
    '7_edge_left': {
        title: "Курцер Оберхау Линкс",
        desc: "Быстрый щелчок короткой стороной сверху-слева."
    },
    '7_edge_right': {
        title: "Линкер Оберхау (Linker Oberhau)",
        desc: "Классический диагональный рубящий удар сверху-слева длинной стороной."
    },

    // THRUSTS (Направленные уколы из 8 стоек)
    'stich_0_center': {
        title: "Верхний укол с оппозицией (Oberstich mit Bindung)",
        desc: "Падающий укол сверху вниз со связыванием и перекрытием центральной линии."
    },
    'stich_0_edge': {
        title: "Шайтель-Штих (Scheitel-Stich)",
        desc: "Прямой вертикальный укол сверху вниз без связывания."
    },

    'stich_1_center': {
        title: "Укол из правого Окса с оппозицией (Ochs-Stich mit Bindung)",
        desc: "Верхний укол справа из стойки Бык с отбивом вражеского клинка и прикрытием головы."
    },
    'stich_1_edge': {
        title: "Рехтер Окс-Штих (Rechter Ochs-Stich)",
        desc: "Прямой выпад уколом из верхней правой стойки Бык."
    },

    'stich_2_center': {
        title: "Боковой укол с оппозицией (Mittelstich mit Bindung)",
        desc: "Поперечный укол справа со связыванием оружия соперника."
    },
    'stich_2_edge': {
        title: "Рехтер Миттельштих (Mittelstich)",
        desc: "Прямой укол сбоку справа по корпусу."
    },

    'stich_3_center': {
        title: "Укол из правого Пфлюга с оппозицией (Pflug-Stich mit Bindung)",
        desc: "Восходящий укол снизу-справа из стойки Плуг с отбивом и отводом клинка."
    },
    'stich_3_edge': {
        title: "Рехтер Пфлюг-Штих (Rechter Pflug-Stich)",
        desc: "Прямой выпад уколом из нижней правой стойки Плуг."
    },

    'stich_4_center': {
        title: "Укол из Альбера с оппозицией (Alber-Stich mit Bindung)",
        desc: "Восходящий вертикальный укол снизу со связыванием и прикрытием центра."
    },
    'stich_4_edge': {
        title: "Герадер Альбер-Штих (Alber-Stich)",
        desc: "Прямой укол снизу вверх из нижней стойки Альбера."
    },

    'stich_5_center': {
        title: "Укол из левого Пфлюга с оппозицией (Pflug-Stich Links mit Bindung)",
        desc: "Восходящий укол снизу-слева из стойки Плуг с отбивом и защитой."
    },
    'stich_5_edge': {
        title: "Линкер Пфлюг-Штих (Linker Pflug-Stich)",
        desc: "Прямой выпад уколом из нижней левой стойки Плуг."
    },

    'stich_6_center': {
        title: "Боковой укол слева с оппозицией (Mittelstich Links mit Bindung)",
        desc: "Поперечный боковой укол слева со связыванием."
    },
    'stich_6_edge': {
        title: "Линкер Миттельштих (Mittelstich Links)",
        desc: "Прямой боковой укол слева."
    },

    'stich_7_center': {
        title: "Укол из левого Окса с оппозицией (Ochs-Stich Links mit Bindung)",
        desc: "Верхний укол слева со скрещиванием рук и прикрытием головы."
    },
    'stich_7_edge': {
        title: "Линкер Окс-Штих (Linker Ochs-Stich)",
        desc: "Прямой выпад уколом из верхней левой стойки Бык."
    }
};

// SVG Element References
const svg = document.getElementById('wheel-svg');
const statusDir = document.getElementById('status-direction');
const statusOpp = document.getElementById('status-opposition');
const statusEdge = document.getElementById('status-edge');
const statusTitle = document.getElementById('status-title');
const statusDesc = document.getElementById('status-desc');
const btnReset = document.getElementById('btn-reset');

// Initialize application
function init() {
    svg.addEventListener('click', handleSvgClick);
    btnReset.addEventListener('click', resetSelection);
    render();
}

// Convert polar to cartesian coordinates (0 deg is UP)
function polarToCartesian(centerX, centerY, radius, angleInDegrees) {
    const angleInRadians = (angleInDegrees - 90) * Math.PI / 180.0;
    return {
        x: centerX + (radius * Math.cos(angleInRadians)),
        y: centerY + (radius * Math.sin(angleInRadians))
    };
}

// Find closest ray for any given angle (0..359)
function getClosestRay(angleDeg) {
    let minDiff = 360;
    let closestIndex = 0;
    
    RAY_ANGLES.forEach((rayAngle, index) => {
        let diff = Math.abs(angleDeg - rayAngle);
        if (diff > 180) diff = 360 - diff;
        if (diff < minDiff) {
            minDiff = diff;
            closestIndex = index;
        }
    });

    return { index: closestIndex, angleDiff: minDiff };
}

// Handle all clicks on the SVG canvas
function handleSvgClick(event) {
    const rect = svg.getBoundingClientRect();
    const clickX = (event.clientX - rect.left) * (600 / rect.width);
    const clickY = (event.clientY - rect.top) * (600 / rect.height);

    const dx = clickX - CX;
    const dy = clickY - CY;
    const distance = Math.sqrt(dx * dx + dy * dy);

    // Click outside outer wheel bounds resets selection
    if (distance > R_OUTER + 30) {
        resetSelection();
        return;
    }

    // 1-ST CLICK ON CENTER CIRCLE: ACTIVATES / TOGGLES THRUST MODE
    if (distance <= R_INNER) {
        if (state.selectedRay === 'center') {
            // Re-clicking center toggles thrust manner: 'active' (точка ●) <-> 'interception' (крестик ┼)
            state.thrustManner = (state.thrustManner === 'active') ? 'interception' : 'active';
        } else {
            state.selectedRay = 'center';
            state.thrustManner = 'active'; // default is active dot ●
            state.thrustDirection = null;
            state.opposition = null;
            state.edge = null;
        }
        render();
        return;
    }

    // RAY CLICKS
    let rawAngle = Math.atan2(dy, dx) * 180 / Math.PI;
    let clickAngle = (rawAngle + 90 + 360) % 360;
    const closestRayInfo = getClosestRay(clickAngle);

    // MODE 1: THRUST MODE ACTIVE (selectedRay === 'center')
    if (state.selectedRay === 'center') {
        state.thrustDirection = closestRayInfo.index;
        let distRatio = Math.min(Math.max((distance - R_INNER) / (R_OUTER - R_INNER), 0), 1);
        
        state.opposition = (distRatio < 0.5) ? 'center' : 'edge';
        state.edge = 'center'; // Symmetrical centered shape for thrusts

        render();
        return;
    }

    // MODE 2: CUT MODE (Normal 8 Rays)
    if (state.selectedRay === null) {
        state.selectedRay = closestRayInfo.index;
        state.opposition = null;
        state.edge = null;
        render();
        return;
    }

    // RE-SELECTION CHECK FOR CUT MODE
    if (closestRayInfo.index !== state.selectedRay && closestRayInfo.angleDiff <= 18) {
        state.selectedRay = closestRayInfo.index;
        state.opposition = null;
        state.edge = null;
        render();
        return;
    }

    // 2nd Click on active cut ray
    const activeAngle = RAY_ANGLES[state.selectedRay];
    let diffAngle = clickAngle - activeAngle;
    while (diffAngle > 180) diffAngle -= 360;
    while (diffAngle < -180) diffAngle += 360;

    let distRatio = Math.min(Math.max((distance - R_INNER) / (R_OUTER - R_INNER), 0), 1);
    
    state.opposition = (distRatio < 0.5) ? 'center' : 'edge';
    state.edge = (diffAngle <= 0) ? 'left' : 'right';

    render();
}

function resetSelection() {
    state.selectedRay = null;
    state.thrustDirection = null;
    state.thrustManner = 'active';
    state.opposition = null;
    state.edge = null;
    render();
}

// Render SVG and Update UI Status
function render() {
    svg.innerHTML = '';

    // Defs for gradients & glow filters
    const defs = document.createElementNS('http://www.w3.org/2000/svg', 'defs');
    
    // Smooth glow filter
    const filter = document.createElementNS('http://www.w3.org/2000/svg', 'filter');
    filter.setAttribute('id', 'glow');
    filter.setAttribute('x', '-30%');
    filter.setAttribute('y', '-30%');
    filter.setAttribute('width', '160%');
    filter.setAttribute('height', '160%');
    filter.innerHTML = `
        <feGaussianBlur stdDeviation="4" result="blur" />
        <feComposite in="SourceGraphic" in2="blur" operator="over" />
    `;
    defs.appendChild(filter);

    // Linear gradient for CUT fill (Ice Blue / Cyan)
    const grad = document.createElementNS('http://www.w3.org/2000/svg', 'linearGradient');
    grad.setAttribute('id', 'bladeGrad');
    grad.setAttribute('x1', '0%');
    grad.setAttribute('y1', '100%');
    grad.setAttribute('x2', '0%');
    grad.setAttribute('y2', '0%');
    grad.innerHTML = `
        <stop offset="0%" stop-color="#0284c7" stop-opacity="0.85" />
        <stop offset="50%" stop-color="#38bdf8" stop-opacity="0.95" />
        <stop offset="100%" stop-color="#e0f2fe" stop-opacity="1" />
    `;
    defs.appendChild(grad);

    // Linear gradient for THRUST fill (Fiery Amber / Gold / Crimson)
    const thrustGrad = document.createElementNS('http://www.w3.org/2000/svg', 'linearGradient');
    thrustGrad.setAttribute('id', 'thrustGrad');
    thrustGrad.setAttribute('x1', '0%');
    thrustGrad.setAttribute('y1', '100%');
    thrustGrad.setAttribute('x2', '0%');
    thrustGrad.setAttribute('y2', '0%');
    thrustGrad.innerHTML = `
        <stop offset="0%" stop-color="#c2410c" stop-opacity="0.9" />
        <stop offset="50%" stop-color="#f97316" stop-opacity="0.95" />
        <stop offset="100%" stop-color="#fef08a" stop-opacity="1" />
    `;
    defs.appendChild(thrustGrad);

    svg.appendChild(defs);

    // 1. Draw Background Circles
    const bgOuter = document.createElementNS('http://www.w3.org/2000/svg', 'circle');
    bgOuter.setAttribute('cx', CX);
    bgOuter.setAttribute('cy', CY);
    bgOuter.setAttribute('r', R_OUTER);
    bgOuter.setAttribute('fill', 'none');
    bgOuter.setAttribute('stroke', '#232d42');
    bgOuter.setAttribute('stroke-width', '2');
    svg.appendChild(bgOuter);

    // Central Circle - Glowing bright when active dot ●, or faded/dim when cross ┼!
    const bgInner = document.createElementNS('http://www.w3.org/2000/svg', 'circle');
    bgInner.setAttribute('cx', CX);
    bgInner.setAttribute('cy', CY);
    bgInner.setAttribute('r', R_INNER);
    bgInner.setAttribute('fill', state.selectedRay === 'center' ? '#1c1917' : '#151c2c');
    
    if (state.selectedRay === 'center') {
        if (state.thrustManner === 'interception') {
            // Faded/dim inner circle stroke when Cross ┼ is active!
            bgInner.setAttribute('stroke', '#f97316');
            bgInner.setAttribute('stroke-width', '2');
            bgInner.setAttribute('opacity', '0.35');
        } else {
            // Bright glowing inner circle stroke when Dot ● is active!
            bgInner.setAttribute('stroke', '#f97316');
            bgInner.setAttribute('stroke-width', '3.5');
            bgInner.setAttribute('opacity', '1');
            bgInner.setAttribute('filter', 'url(#glow)');
        }
    } else {
        bgInner.setAttribute('stroke', '#232d42');
        bgInner.setAttribute('stroke-width', '2');
    }
    svg.appendChild(bgInner);

    // Center Indicator: Dot ● for Active Thrust, Cross ┼ for Interception (Набежал сам)
    if (state.selectedRay === 'center' && state.thrustManner === 'interception') {
        // Draw Orange Cross ┼ at center
        const crossGroup = document.createElementNS('http://www.w3.org/2000/svg', 'g');
        crossGroup.setAttribute('filter', 'url(#glow)');
        
        const lineH = document.createElementNS('http://www.w3.org/2000/svg', 'line');
        lineH.setAttribute('x1', CX - 9);
        lineH.setAttribute('y1', CY);
        lineH.setAttribute('x2', CX + 9);
        lineH.setAttribute('y2', CY);
        lineH.setAttribute('stroke', '#f97316');
        lineH.setAttribute('stroke-width', '3.5');
        lineH.setAttribute('stroke-linecap', 'round');

        const lineV = document.createElementNS('http://www.w3.org/2000/svg', 'line');
        lineV.setAttribute('x1', CX);
        lineV.setAttribute('y1', CY - 9);
        lineV.setAttribute('x2', CX);
        lineV.setAttribute('y2', CY + 9);
        lineV.setAttribute('stroke', '#f97316');
        lineV.setAttribute('stroke-width', '3.5');
        lineV.setAttribute('stroke-linecap', 'round');

        crossGroup.appendChild(lineH);
        crossGroup.appendChild(lineV);
        svg.appendChild(crossGroup);
    } else {
        // Draw Orange Dot ● at center
        const centerDot = document.createElementNS('http://www.w3.org/2000/svg', 'circle');
        centerDot.setAttribute('cx', CX);
        centerDot.setAttribute('cy', CY);
        centerDot.setAttribute('r', '7');
        centerDot.setAttribute('fill', '#f97316');
        if (state.selectedRay === 'center') {
            centerDot.setAttribute('filter', 'url(#glow)');
        }
        svg.appendChild(centerDot);
    }

    // 2. Draw 8 Rays with subtle theme colors for inactive rays
    RAY_ANGLES.forEach((angle, index) => {
        const start = polarToCartesian(CX, CY, R_INNER, angle);
        const end = polarToCartesian(CX, CY, R_OUTER, angle);

        const line = document.createElementNS('http://www.w3.org/2000/svg', 'line');
        line.setAttribute('x1', start.x);
        line.setAttribute('y1', start.y);
        line.setAttribute('x2', end.x);
        line.setAttribute('y2', end.y);

        let lineClass = 'ray-line';

        if (state.selectedRay === 'center') {
            if (state.thrustDirection === null) {
                // Awaiting thrust direction: ALL 8 RAYS LIGHT UP SUBTLY ORANGE!
                lineClass += ' thrust-pending';
            } else if (state.thrustDirection === index) {
                // Selected thrust direction: BRIGHT ACTIVE ORANGE!
                lineClass += ' thrust-active';
            } else {
                // OTHER 7 RAYS STAY SUBTLY ORANGE!
                lineClass += ' thrust-faded';
            }
        } else if (state.selectedRay !== null) {
            if (state.selectedRay === index) {
                // Selected cut direction: BRIGHT ACTIVE BLUE!
                lineClass += ' cut-active';
            } else {
                // OTHER 7 RAYS STAY SUBTLY BLUE!
                lineClass += ' cut-faded';
            }
        } else {
            lineClass += ' idle';
        }
        line.setAttribute('class', lineClass);
        svg.appendChild(line);
    });

    // 3. Draw Dynamic Elements
    if (state.selectedRay === 'center') {
        // THRUST MODE: 2 Side Corners touch the Inner Circle Radius (R_INNER = 55) EXACTLY!
        if (state.thrustDirection !== null && state.opposition !== null) {
            const rayAngle = RAY_ANGLES[state.thrustDirection];
            const g = document.createElementNS('http://www.w3.org/2000/svg', 'g');
            g.setAttribute('transform', `rotate(${rayAngle}, ${CX}, ${CY})`);

            const yBase = CY; // Starts at center!
            const yTip = CY - R_OUTER;

            // Two side corners touch inner circle radius R_INNER (55px) precisely!
            // Polar distance = sqrt(24^2 + 49.38^2) = 55.0px (Exact alignment with inner circle arc!)
            const wingX = 24;
            const wingY = (state.opposition === 'center')
                ? CY - 49.4  // Touches R_INNER = 55.0px exactly!
                : CY - (R_INNER + 0.55 * (R_OUTER - R_INNER));

            const cp1Left = CX - wingX * 0.35;
            const cp1Right = CX + wingX * 0.35;

            const pathData = `
                M ${CX} ${yBase}
                Q ${cp1Left} ${(yBase + wingY) / 2} ${CX - wingX} ${wingY}
                Q ${cp1Left} ${(wingY + yTip) / 2} ${CX} ${yTip}
                Q ${cp1Right} ${(wingY + yTip) / 2} ${CX + wingX} ${wingY}
                Q ${cp1Right} ${(yBase + wingY) / 2} ${CX} ${yBase}
                Z
            `;

            const path = document.createElementNS('http://www.w3.org/2000/svg', 'path');
            path.setAttribute('d', pathData);
            path.setAttribute('fill', 'url(#thrustGrad)');
            path.setAttribute('stroke', '#ffffff');
            path.setAttribute('stroke-width', '2');
            path.setAttribute('stroke-linejoin', 'round');
            path.setAttribute('stroke-linecap', 'round');
            path.setAttribute('filter', 'url(#glow)');
            g.appendChild(path);

            svg.appendChild(g);
        }
    } else if (state.selectedRay !== null) {
        // CUT MODE: Draw Asymmetrical Blade Shape
        const rayAngle = RAY_ANGLES[state.selectedRay];
        const g = document.createElementNS('http://www.w3.org/2000/svg', 'g');
        g.setAttribute('transform', `rotate(${rayAngle}, ${CX}, ${CY})`);

        if (state.opposition === null || state.edge === null) {
            const guidePath = document.createElementNS('http://www.w3.org/2000/svg', 'path');
            const midY = CY - (R_INNER + 0.5 * (R_OUTER - R_INNER));
            guidePath.setAttribute('d', `M ${CX - 35} ${midY} L ${CX + 35} ${midY}`);
            guidePath.setAttribute('stroke', '#38bdf8');
            guidePath.setAttribute('stroke-width', '1.5');
            guidePath.setAttribute('stroke-dasharray', '3,3');
            guidePath.setAttribute('opacity', '0.4');
            g.appendChild(guidePath);
        } else {
            const yBase = CY - R_INNER;
            const yTip = CY - R_OUTER;

            const yPeak = (state.opposition === 'center')
                ? CY - (R_INNER + 0.32 * (R_OUTER - R_INNER))
                : CY - (R_INNER + 0.75 * (R_OUTER - R_INNER));

            const sideSign = (state.edge === 'left') ? -1 : 1;
            const maxWingWidth = 48 * sideSign;

            const cp1X = CX + maxWingWidth * 0.25;
            const cp1Y = (yBase + yPeak) / 2;

            const cp2X = CX + maxWingWidth * 0.25;
            const cp2Y = (yPeak + yTip) / 2;

            const pathData = `
                M ${CX} ${yBase}
                Q ${cp1X} ${cp1Y} ${CX + maxWingWidth} ${yPeak}
                Q ${cp2X} ${cp2Y} ${CX} ${yTip}
                Z
            `;

            const path = document.createElementNS('http://www.w3.org/2000/svg', 'path');
            path.setAttribute('d', pathData);
            path.setAttribute('fill', 'url(#bladeGrad)');
            path.setAttribute('stroke', '#ffffff');
            path.setAttribute('stroke-width', '2.5');
            path.setAttribute('stroke-linejoin', 'round');
            path.setAttribute('stroke-linecap', 'round');
            path.setAttribute('filter', 'url(#glow)');
            g.appendChild(path);
        }

        svg.appendChild(g);
    }

    // 4. Update UI Status Cards
    updateStatusUI();
}

function updateStatusUI() {
    if (state.selectedRay === 'center') {
        const mannerLabel = (state.thrustManner === 'interception') ? ' (Встречный / Набежал сам)' : ' (Доведенный выпад)';
        if (state.thrustDirection !== null) {
            statusDir.textContent = `Укол -> ${RAY_NAMES[state.thrustDirection]}${mannerLabel}`;
        } else {
            statusDir.textContent = `Укол${mannerLabel} (Выберите луч)`;
        }
        statusDir.className = 'value active';
        statusDir.style.color = '#f97316';
    } else if (state.selectedRay !== null) {
        statusDir.textContent = `Удар -> ${RAY_NAMES[state.selectedRay]}`;
        statusDir.className = 'value active';
        statusDir.style.color = '#38bdf8';
    } else {
        statusDir.textContent = 'Не выбрано';
        statusDir.className = 'value empty';
    }

    if (state.opposition !== null) {
        statusOpp.textContent = (state.opposition === 'center') ? 'Взял оппозицию' : 'Не взял оппозицию';
        statusOpp.className = 'value active';
    } else {
        statusOpp.textContent = 'Сделайте 2-й клик...';
        statusOpp.className = 'value empty';
    }

    if (state.selectedRay === 'center') {
        statusEdge.textContent = (state.thrustManner === 'interception') ? 'Встречный заслон (Absetzen)' : 'Прямой выпад острием';
        statusEdge.className = 'value active';
    } else if (state.edge !== null) {
        statusEdge.textContent = (state.edge === 'left') ? 'Короткая сторона' : 'Длинная сторона';
        statusEdge.className = 'value active';
    } else {
        statusEdge.textContent = 'Сделайте 2-й клик...';
        statusEdge.className = 'value empty';
    }

    // Database Lookup for HEMA Technique
    if (state.selectedRay === 'center' && state.thrustDirection !== null && state.opposition !== null) {
        const key = `stich_${state.thrustDirection}_${state.opposition}`;
        const tech = TECHNIQUES[key];
        const mannerPrefix = (state.thrustManner === 'interception') ? 'Встречный ' : '';

        if (tech) {
            statusTitle.textContent = mannerPrefix + tech.title;
            statusTitle.style.color = '#f97316';
            statusDesc.textContent = (state.thrustManner === 'interception') 
                ? 'Наставленный встречный укол (Absetzen): меч выставлен в структуру, противник набежал на клинок сам.'
                : tech.desc;
        } else {
            statusTitle.textContent = mannerPrefix + 'Направленный укол';
            statusDesc.textContent = 'Укол сформирован.';
        }
    } else if (state.selectedRay !== null && state.selectedRay !== 'center' && state.opposition !== null && state.edge !== null) {
        const key = `${state.selectedRay}_${state.opposition}_${state.edge}`;
        const tech = TECHNIQUES[key];

        if (tech) {
            statusTitle.textContent = tech.title;
            statusTitle.style.color = '#4ade80';
            statusDesc.textContent = tech.desc;
        } else {
            statusTitle.textContent = 'Приём определен';
            statusDesc.textContent = 'Комбинация действия сформирована.';
        }
    } else if (state.selectedRay === 'center') {
        statusTitle.textContent = (state.thrustManner === 'interception') ? 'Режим Встречных Уколов (Absetzen)' : 'Режим Выпадов Уколом (Stich)';
        statusTitle.style.color = '#f97316';
        statusDesc.textContent = 'Повторный клик по центру переключает режим (Точка ● = Выпад / Крестик ┼ = Встречный).';
    } else if (state.selectedRay !== null) {
        statusTitle.textContent = 'Режим Ударов (Hau-Modus)';
        statusTitle.style.color = '#38bdf8';
        statusDesc.textContent = 'Кликните на любую дистанцию около выбранного луча.';
    } else {
        statusTitle.textContent = 'Выберите приём';
        statusTitle.style.color = '#94a3b8';
        statusDesc.textContent = 'Клик по лучу = УДАР (Hau). Клик по центру = УКОЛ (Stich).';
    }
}

// Launch application on page load
window.addEventListener('DOMContentLoaded', init);

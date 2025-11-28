document.addEventListener('DOMContentLoaded', () => {
    const sporesContainer = document.getElementById('spores');
    const sporeCount = 50;

    function createSpore() {
        const spore = document.createElement('div');
        spore.classList.add('spore');

        // Random positioning
        const x = Math.random() * 100; // vw
        const delay = Math.random() * 5; // s
        const duration = 5 + Math.random() * 10; // s
        const size = 2 + Math.random() * 4; // px

        spore.style.left = `${x}vw`;
        spore.style.top = '100vh';
        spore.style.width = `${size}px`;
        spore.style.height = `${size}px`;
        spore.style.animationDelay = `${delay}s`;
        spore.style.animationDuration = `${duration}s`;

        sporesContainer.appendChild(spore);

        // Remove spore after animation to prevent DOM clutter (optional, but good for long running)
        // For infinite loop with CSS, we might just let them loop if we set animation iteration count to infinite
        // But the CSS has 'infinite', so we just need to create them once.
        // However, to make it look natural, we might want to stagger their creation or just create them all with random delays.
    }

    for (let i = 0; i < sporeCount; i++) {
        createSpore();
    }

    // Flashlight Effect
    const flashlight = document.getElementById('flashlight');
    document.addEventListener('mousemove', (e) => {
        flashlight.style.setProperty('--x', `${e.clientX}px`);
        flashlight.style.setProperty('--y', `${e.clientY}px`);
    });

    // Terminal Logic
    window.openTerminal = function (type) {
        const terminalContent = document.getElementById('terminal-content');
        const docsSection = document.getElementById('docs-section');

        // Scroll to documentation
        docsSection.scrollIntoView({ behavior: 'smooth' });

        let text = '';
        switch (type) {
            case 'tasks':
                text = `
> ACCESSING TASK_LOGS...
> DECRYPTING...
> 
> [MISSION OBJECTIVES]
> --------------------
> 1. LOCATE WILL BYERS
> 2. CLOSE THE GATE
> 3. BUY EGGO WAFFLES
> 
> STATUS: PENDING
> PRIORITY: CRITICAL
`;
                break;
            case 'events':
                text = `
> ACCESSING EVENT_LOGS...
> DECRYPTING...
> 
> [TIMELINE ANOMALIES]
> --------------------
> NOV 06 1983: DISAPPEARANCE OF W.B.
> NOV 07 1983: BENNY'S BURGERS INCIDENT
> NOV 12 1983: BODY FOUND (FAKE)
> 
> WARNING: DEMOGORGON SIGHTINGS CONFIRMED.
`;
                break;
            case 'notes':
                text = `
> ACCESSING RESEARCH_NOTES...
> DECRYPTING...
> 
> [SUBJECT: ELEVEN]
> -----------------
> POWERS: TELEKINESIS, REMOTE VIEWING
> DIET: WAFFLES
> STATE: UNSTABLE
> 
> NOTE: KEEP THE DOOR OPEN 3 INCHES.
`;
                break;
        }

        // Typewriter effect
        terminalContent.innerHTML = '<div class="line">> <span class="cursor">_</span></div>';
        let i = 0;
        const speed = 30;

        // Clear previous interval if any (simple implementation)
        // Ideally we'd track the interval ID.

        terminalContent.innerHTML = '';

        function typeWriter() {
            if (i < text.length) {
                const char = text.charAt(i);
                if (char === '\n') {
                    terminalContent.innerHTML += '<br>';
                } else {
                    terminalContent.innerHTML += char;
                }
                i++;
                setTimeout(typeWriter, speed);
            } else {
                terminalContent.innerHTML += '<br><br>> <a href="https://github.com/ankur-gupta-29/bullet-journal" target="_blank" style="color: inherit; text-decoration: underline;">[ACCESS_FULL_ARCHIVES]</a>';
                terminalContent.innerHTML += '<div class="line">> <span class="cursor">_</span></div>';
            }
        }

        typeWriter();
    };

    // Random Lightning
    function triggerLightning() {
        if (document.body.classList.contains('upside-down')) {
            const vignette = document.querySelector('.vignette');
            vignette.style.animation = 'none';
            vignette.offsetHeight; /* trigger reflow */
            vignette.style.animation = `lightning ${Math.random() * 3 + 2}s infinite`;
        }
        setTimeout(triggerLightning, Math.random() * 5000 + 2000);
    }
    triggerLightning();

    // Upside Down Toggle
    const toggleBtn = document.getElementById('upside-down-btn');
    let audioContext = null;
    let oscillator = null;

    function playDrone() {
        if (!audioContext) {
            audioContext = new (window.AudioContext || window.webkitAudioContext)();
        }
        if (oscillator) {
            oscillator.stop();
            oscillator = null;
            return;
        }

        // Create a low drone sound
        oscillator = audioContext.createOscillator();
        const gainNode = audioContext.createGain();

        oscillator.type = 'sawtooth';
        oscillator.frequency.setValueAtTime(50, audioContext.currentTime);

        // LFO for modulation
        const lfo = audioContext.createOscillator();
        lfo.type = 'sine';
        lfo.frequency.setValueAtTime(0.5, audioContext.currentTime);
        const lfoGain = audioContext.createGain();
        lfoGain.gain.setValueAtTime(20, audioContext.currentTime);

        lfo.connect(lfoGain);
        lfoGain.connect(oscillator.frequency);
        lfo.start();

        gainNode.gain.setValueAtTime(0.05, audioContext.currentTime);

        oscillator.connect(gainNode);
        gainNode.connect(audioContext.destination);
        oscillator.start();
    }

    toggleBtn.addEventListener('click', () => {
        document.body.classList.toggle('upside-down');

        // Toggle Drone Sound
        playDrone();

        if (document.body.classList.contains('upside-down')) {
            toggleBtn.textContent = 'RETURN TO HAWKINS';
            toggleBtn.style.color = '#0066ff';
            toggleBtn.style.borderColor = '#0066ff';
        } else {
            toggleBtn.textContent = 'ENTER THE UPSIDE DOWN';
            toggleBtn.style.color = '';
            toggleBtn.style.borderColor = '';
        }
    });

    // EASTER EGG 1: KONAMI CODE (The Real Upside Down)
    const konamiCode = ['ArrowUp', 'ArrowUp', 'ArrowDown', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'ArrowLeft', 'ArrowRight', 'b', 'a'];
    let konamiIndex = 0;

    document.addEventListener('keydown', (e) => {
        if (e.key === konamiCode[konamiIndex]) {
            konamiIndex++;
            if (konamiIndex === konamiCode.length) {
                activateKonami();
                konamiIndex = 0;
            }
        } else {
            konamiIndex = 0;
        }
    });

    function activateKonami() {
        document.body.style.transition = 'transform 2s ease';
        document.body.style.transform = document.body.style.transform === 'rotate(180deg)' ? 'rotate(0deg)' : 'rotate(180deg)';

        // Play a special sound
        if (!audioContext) {
            audioContext = new (window.AudioContext || window.webkitAudioContext)();
        }
        const osc = audioContext.createOscillator();
        const gain = audioContext.createGain();
        osc.type = 'sine';
        osc.frequency.setValueAtTime(880, audioContext.currentTime); // High pitch
        osc.frequency.exponentialRampToValueAtTime(110, audioContext.currentTime + 1); // Drop
        gain.gain.setValueAtTime(0.1, audioContext.currentTime);
        gain.gain.exponentialRampToValueAtTime(0.01, audioContext.currentTime + 1);

        osc.connect(gain);
        gain.connect(audioContext.destination);
        osc.start();
        osc.stop(audioContext.currentTime + 1);
    }

    // EASTER EGG 2: TYPE "11" (Nose Bleed)
    let elevenSequence = '';
    document.addEventListener('keydown', (e) => {
        if (e.key === '1') {
            elevenSequence += '1';
            if (elevenSequence === '11') {
                triggerNoseBleed();
                elevenSequence = '';
            }
        } else {
            elevenSequence = '';
        }
    });

    function triggerNoseBleed() {
        const bleed = document.createElement('div');
        bleed.classList.add('nose-bleed');
        document.body.appendChild(bleed);

        setTimeout(() => {
            bleed.remove();
        }, 5000);
    }

    // EASTER EGG 3: "RUN" LIGHTS MESSAGE
    // Occasionally flash lights to spell RUN
    function runMessage() {
        const lights = document.querySelectorAll('.light');
        if (lights.length === 0) return;

        // Indices for R, U, N (simulated positions)
        const rIndex = 4;
        const uIndex = 8;
        const nIndex = 12;

        function flash(index, delay) {
            setTimeout(() => {
                lights[index].style.filter = 'brightness(3) drop-shadow(0 0 20px currentColor)';
                setTimeout(() => {
                    lights[index].style.filter = '';
                }, 800);
            }, delay);
        }

        // Sequence: R ... U ... N
        flash(rIndex, 0);
        flash(uIndex, 1000);
        flash(nIndex, 2000);

        // Repeat randomly
        setTimeout(runMessage, Math.random() * 20000 + 10000);
    }

    // Start message loop after a delay
    setTimeout(runMessage, 5000);
});

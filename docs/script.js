document.addEventListener('DOMContentLoaded', () => {
    // Register Service Worker for PWA
    if ('serviceWorker' in navigator) {
        navigator.serviceWorker.register('sw.js');
    }

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

    // Light Wall Generation
    const lightWall = document.getElementById('light-wall');
    const alphabet = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ';
    const colors = ['#ff0000', '#00ff00', '#ffff00', '#0000ff']; // Red, Green, Yellow, Blue

    const lightMap = {};

    alphabet.split('').forEach((letter, index) => {
        const container = document.createElement('div');
        container.classList.add('light-bulb-container');

        const bulb = document.createElement('div');
        bulb.classList.add('light-bulb');
        bulb.dataset.color = colors[index % colors.length];

        const char = document.createElement('span');
        char.classList.add('light-letter');
        char.innerText = letter;

        container.appendChild(bulb);
        container.appendChild(char);
        lightWall.appendChild(container);

        lightMap[letter] = bulb;
    });

    // Communication Logic
    const commInput = document.getElementById('comm-input');
    const commBtn = document.getElementById('comm-btn');

    function transmitMessage(message) {
        if (!message) return;

        let i = 0;
        const speed = 1000; // 1 second per letter

        function flashNext() {
            if (i < message.length) {
                const char = message[i].toUpperCase();
                if (lightMap[char]) {
                    const bulb = lightMap[char];
                    const originalColor = bulb.dataset.color;

                    bulb.style.backgroundColor = originalColor;
                    bulb.classList.add('active');

                    // Sound
                    if (!audioContext) audioContext = new (window.AudioContext || window.webkitAudioContext)();
                    const osc = audioContext.createOscillator();
                    const gain = audioContext.createGain();
                    osc.type = 'sine';
                    osc.frequency.setValueAtTime(300 + (char.charCodeAt(0) * 10), audioContext.currentTime);
                    gain.gain.setValueAtTime(0.1, audioContext.currentTime);
                    gain.gain.exponentialRampToValueAtTime(0.01, audioContext.currentTime + 0.5);
                    osc.connect(gain);
                    gain.connect(audioContext.destination);
                    osc.start();
                    osc.stop(audioContext.currentTime + 0.5);

                    setTimeout(() => {
                        bulb.style.backgroundColor = '#444';
                        bulb.classList.remove('active');
                    }, 800);
                }
                i++;
                setTimeout(flashNext, speed);
            }
        }
        flashNext();
    }

    commBtn.addEventListener('click', () => {
        transmitMessage(commInput.value);
        commInput.value = '';
    });

    commInput.addEventListener('keypress', (e) => {
        if (e.key === 'Enter') {
            transmitMessage(commInput.value);
            commInput.value = '';
        }
    });

    // EASTER EGG 3: "RUN" LIGHTS MESSAGE (Updated for new wall)
    // Occasionally flash lights to spell RUN
    function runMessage() {
        transmitMessage('RUN');
        // Repeat randomly
        setTimeout(runMessage, Math.random() * 20000 + 10000);
    }

    // Start message loop after a delay
    setTimeout(runMessage, 5000);

    // EASTER EGG 4: TYPE "WAFFLE" (Eleven's Favorite)
    const waffleCode = 'waffle';
    let waffleIndex = 0;

    document.addEventListener('keydown', (e) => {
        if (e.key.toLowerCase() === waffleCode[waffleIndex]) {
            waffleIndex++;
            if (waffleIndex === waffleCode.length) {
                triggerWaffleRain();
                waffleIndex = 0;
            }
        } else {
            waffleIndex = 0;
            // Retry if the current key matches the first letter
            if (e.key.toLowerCase() === waffleCode[0]) {
                waffleIndex = 1;
            }
        }
    });

    function triggerWaffleRain() {
        for (let i = 0; i < 20; i++) {
            setTimeout(() => {
                const waffle = document.createElement('div');
                waffle.classList.add('waffle');
                waffle.style.left = Math.random() * 100 + 'vw';
                waffle.style.animationDuration = (Math.random() * 2 + 2) + 's';
                document.body.appendChild(waffle);

                setTimeout(() => {
                    waffle.remove();
                }, 4000);
            }, i * 200);
        }
    }

    // EASTER EGG 5: CLICK TITLE (Demogorgon Roar)
    const title = document.querySelector('.stranger-title');
    title.style.cursor = 'pointer';
    title.addEventListener('click', () => {
        document.body.classList.add('shake');

        // Play Roar Sound
        if (!audioContext) {
            audioContext = new (window.AudioContext || window.webkitAudioContext)();
        }

        // Noise buffer for roar
        const bufferSize = audioContext.sampleRate * 1.5; // 1.5 seconds
        const buffer = audioContext.createBuffer(1, bufferSize, audioContext.sampleRate);
        const data = buffer.getChannelData(0);

        for (let i = 0; i < bufferSize; i++) {
            data[i] = Math.random() * 2 - 1;
        }

        const noise = audioContext.createBufferSource();
        noise.buffer = buffer;

        const noiseFilter = audioContext.createBiquadFilter();
        noiseFilter.type = 'lowpass';
        noiseFilter.frequency.value = 1000;

        const noiseGain = audioContext.createGain();
        noiseGain.gain.setValueAtTime(0.5, audioContext.currentTime);
        noiseGain.gain.exponentialRampToValueAtTime(0.01, audioContext.currentTime + 1.5);

        noise.connect(noiseFilter);
        noiseFilter.connect(noiseGain);
        noiseGain.connect(audioContext.destination);
        noise.start();

        setTimeout(() => {
            document.body.classList.remove('shake');
        }, 500);
    });

    // EASTER EGG 6: TYPE "MAX" (Levitation)
    const maxCode = 'max';
    let maxIndex = 0;

    document.addEventListener('keydown', (e) => {
        if (e.key.toLowerCase() === maxCode[maxIndex]) {
            maxIndex++;
            if (maxIndex === maxCode.length) {
                triggerMaxLevitation();
                maxIndex = 0;
            }
        } else {
            maxIndex = 0;
            if (e.key.toLowerCase() === maxCode[0]) maxIndex = 1;
        }
    });

    function triggerMaxLevitation() {
        const cards = document.querySelectorAll('.feature-card');
        cards.forEach((card, index) => {
            setTimeout(() => {
                card.classList.add('levitate');
            }, index * 200);
        });

        // Stop after 8 seconds (song duration-ish)
        setTimeout(() => {
            cards.forEach(card => {
                card.classList.remove('levitate');
            });
        }, 8000);
    }

    // EASTER EGG 7: TYPE "DND" (Hellfire Club)
    const dndCode = 'dnd';
    let dndIndex = 0;

    document.addEventListener('keydown', (e) => {
        if (e.key.toLowerCase() === dndCode[dndIndex]) {
            dndIndex++;
            if (dndIndex === dndCode.length) {
                triggerCriticalHit();
                dndIndex = 0;
            }
        } else {
            dndIndex = 0;
            if (e.key.toLowerCase() === dndCode[0]) dndIndex = 1;
        }
    });

    function triggerCriticalHit() {
        const d20 = document.createElement('div');
        d20.classList.add('d20-overlay');
        document.body.appendChild(d20);

        // Play dice roll sound (simulated)
        if (!audioContext) {
            audioContext = new (window.AudioContext || window.webkitAudioContext)();
        }

        // Quick click/thud sound
        const osc = audioContext.createOscillator();
        const gain = audioContext.createGain();
        osc.type = 'square';
        osc.frequency.setValueAtTime(100, audioContext.currentTime);
        osc.frequency.exponentialRampToValueAtTime(0.01, audioContext.currentTime + 0.1);
        gain.gain.setValueAtTime(0.5, audioContext.currentTime);
        gain.gain.exponentialRampToValueAtTime(0.01, audioContext.currentTime + 0.1);

        osc.connect(gain);
        gain.connect(audioContext.destination);
        osc.start();
        osc.stop(audioContext.currentTime + 0.1);

        setTimeout(() => {
            d20.remove();
        }, 3000);
    }
    // EASTER EGG 8: VECNA'S CURSE (Type "vecna")
    // WARNING: THIS WILL "DESTROY" THE WEBSITE
    const vecnaCode = 'vecna';
    let vecnaIndex = 0;

    document.addEventListener('keydown', (e) => {
        if (e.key.toLowerCase() === vecnaCode[vecnaIndex]) {
            vecnaIndex++;
            if (vecnaIndex === vecnaCode.length) {
                triggerVecnaCurse();
                vecnaIndex = 0;
            }
        } else {
            vecnaIndex = 0;
            if (e.key.toLowerCase() === vecnaCode[0]) vecnaIndex = 1;
        }
    });

    function triggerVecnaCurse() {
        // 1. Play Clock Chime Sound (Ominous)
        if (!audioContext) audioContext = new (window.AudioContext || window.webkitAudioContext)();

        // Create a deep, scary drone
        const osc = audioContext.createOscillator();
        const gain = audioContext.createGain();
        osc.type = 'sawtooth';
        osc.frequency.setValueAtTime(50, audioContext.currentTime);
        osc.frequency.exponentialRampToValueAtTime(10, audioContext.currentTime + 4);
        gain.gain.setValueAtTime(0.5, audioContext.currentTime);
        gain.gain.linearRampToValueAtTime(0, audioContext.currentTime + 4);

        osc.connect(gain);
        gain.connect(audioContext.destination);
        osc.start();
        osc.stop(audioContext.currentTime + 4);

        // 2. Visual Chaos
        document.body.classList.add('vecna-curse');

        // Add Veins and Fog
        const veins = document.createElement('div');
        veins.classList.add('vecna-veins');
        document.body.appendChild(veins);

        const fog = document.createElement('div');
        fog.classList.add('vecna-fog');
        document.body.appendChild(fog);

        // Add Mind Hive (Swirling Particles)
        const hive = document.createElement('div');
        hive.classList.add('vecna-hive-mind');
        document.body.appendChild(hive);

        for (let i = 0; i < 100; i++) {
            const particle = document.createElement('div');
            particle.classList.add('hive-particle');
            particle.style.setProperty('--start-x', (Math.random() * 100 - 50) + 'vw');
            particle.style.setProperty('--start-y', (Math.random() * 100 - 50) + 'vh');
            particle.style.setProperty('--end-x', (Math.random() * 100 - 50) + 'vw');
            particle.style.setProperty('--end-y', (Math.random() * 100 - 50) + 'vh');
            particle.style.animationDelay = Math.random() * 5 + 's';
            hive.appendChild(particle);
        }

        // Fade them in
        setTimeout(() => {
            veins.style.opacity = '1';
            fog.style.opacity = '0.8';
        }, 100);

        // 3. "Crack" the screen (CSS overlay)
        const crack = document.createElement('div');
        crack.classList.add('screen-crack');
        document.body.appendChild(crack);

        // Crack appears suddenly
        setTimeout(() => {
            crack.style.opacity = '1';
        }, 3000);

        // 4. Delete Elements one by one
        // IMPORTANT: We must NOT delete the new atmospheric elements we just added!
        const protectedClasses = ['screen-crack', 'vecna-curse', 'vecna-veins', 'vecna-fog', 'vecna-hive-mind', 'hive-particle'];

        const elements = Array.from(document.body.children).filter(el => {
            // Check if element has any of the protected classes
            for (const cls of protectedClasses) {
                if (el.classList.contains(cls)) return false;
            }
            return true;
        });

        elements.forEach((el, index) => {
            setTimeout(() => {
                el.style.transition = 'transform 0.5s, opacity 0.5s';
                el.style.transform = `rotate(${Math.random() * 90 - 45}deg) scale(0)`;
                el.style.opacity = '0';
            }, index * 200 + 1000); // Start deleting after 1s
        });

        // 5. Final Message
        setTimeout(() => {
            // Instead of clearing innerHTML (which kills our effects), let's just hide the deleted elements permanently
            // and append the message on top.

            // Create a container for the message that sits on top of everything
            const msgContainer = document.createElement('div');
            msgContainer.style.position = 'fixed';
            msgContainer.style.top = '0';
            msgContainer.style.left = '0';
            msgContainer.style.width = '100%';
            msgContainer.style.height = '100%';
            msgContainer.style.display = 'flex';
            msgContainer.style.flexDirection = 'column';
            msgContainer.style.justifyContent = 'center';
            msgContainer.style.alignItems = 'center';
            msgContainer.style.zIndex = '10000'; // Above everything
            msgContainer.style.backgroundColor = 'rgba(0, 0, 0, 0.5)'; // Add a semi-transparent black background to ensure readability

            document.body.appendChild(msgContainer);

            const msg = document.createElement('h1');
            msg.innerText = "YOUR SUFFERING IS ALMOST AT AN END.";
            msg.style.color = '#ce1010';
            msg.style.fontFamily = "'Playfair Display', serif";
            msg.style.fontSize = '3rem';
            msg.style.textAlign = 'center';
            msg.style.opacity = '0';
            msg.style.transition = 'opacity 2s';

            msgContainer.appendChild(msg); // Append to container, not body

            // Force reflow
            setTimeout(() => { msg.style.opacity = '1'; }, 100);

            // Restore button (to fix the site)
            setTimeout(() => {
                const restoreBtn = document.createElement('button');
                restoreBtn.innerText = "FIGHT BACK (Reload)";
                restoreBtn.style.marginTop = '2rem';
                restoreBtn.style.padding = '10px 20px';
                restoreBtn.style.background = 'transparent';
                restoreBtn.style.border = '1px solid #ce1010';
                restoreBtn.style.color = '#ce1010';
                restoreBtn.style.cursor = 'pointer';
                restoreBtn.style.fontFamily = "'Courier New', monospace";

                restoreBtn.onclick = () => location.reload();

                msgContainer.appendChild(document.createElement('br'));
                msgContainer.appendChild(restoreBtn);
            }, 3000);

        }, elements.length * 200 + 2000);
    }
});

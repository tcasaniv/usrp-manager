use crate::config::AppConfig;
use crate::detection::jtag::{self, find_djtgcfg};
use crate::executor::run_visible;
use crate::images::local::list_local_images;
use crate::ui::prompts::{confirm, input, select_str};
use crate::ui::styles::{dim, err, info, ok, press_enter, print_section, step, warn};
use console::style;
use rust_i18n::t;

pub fn run_jtag_unbrick(config: &AppConfig) {
    print_section(&t!("jtag.title"));
    println!(
        "\n  {}\n  Si un flasheo previo se interrumpió o la imagen se corrompió, el USRP no puede arrancar por sí solo,\n  quedando sin LEDs o sin responder a la red. Esto NO daña el hardware: la FPGA se puede revivir por JTAG.\n",
        style("● ¿Qué significa que tu USRP esté brickeado?").cyan().bold()
    );
    println!(
        "  {}\n  El cable USB JTAG programa la FPGA directamente en memoria volátil (RAM). El equipo volverá a encender\n",
        style("● ¿Cómo funciona la recuperación por JTAG?").cyan().bold()
    );

    // 1. Check Digilent Adept Utilities
    step(&t!("diagnostic.checking_jtag"));
    let djtgcfg = match find_djtgcfg() {
        Some(p) => {
            ok(&format!("Herramienta Digilent encontrada: {}", p.display()));
            p
        }
        None => {
            err(&t!("jtag.no_djtgcfg"));
            println!();
            info(&t!("jtag.install_guide_title"));
            dim("  1. Descarga 'Digilent Adept Runtime' y 'Adept Utilities' desde digilent.com");
            dim("  2. Instálalas y vuelve a ejecutar usrp-manager.");
            press_enter();
            return;
        }
    };

    // 2. Step-by-Step Model Guide with Physical Instructions
    println!();
    println!(
        "  {}\n  Selecciona el modelo de tu USRP para ver cómo conectarlo:",
        style("● Instrucciones Físicas por Modelo").cyan().bold()
    );
    let model_options = vec![
        "USRP N200 / N210 (Botón S2 / Modo Seguro)".to_string(),
        "USRP X300 / X310 (Conector micro-USB frontal JTAG)".to_string(),
        "USRP B200 / B210 (Carga automática por USB 3.0)".to_string(),
        t!("common.back").to_string(),
    ];
    let model_idx = match select_str("Selecciona tu modelo:", &model_options) {
        Ok(i) => i,
        Err(_) => return,
    };
    if model_idx >= model_options.len() - 1 {
        return;
    }

    match model_idx {
        0 => {
            println!();
            println!("{}", style("─── PROCEDIMIENTO DE MODO SEGURO (N200 / N210) ───").yellow().bold());
            println!("  1. Apaga el USRP N210 desconectando el conector de alimentación.");
            println!("  2. {}", style("Mantén presionado el botón S2").bold());
            println!("     (pequeño pulsador junto a los LEDs o en la placa interna) mientras conectas la corriente.");
            println!("  3. Esto fuerza al USRP a arrancar con una imagen segura de respaldo en IP 192.168.10.2.");
            println!("  4. Si aún así no responde, conecta el cable Digilent USB JTAG al puerto JTAG de 6 pines.");
            println!("{}", style("──────────────────────────────────────────────────").yellow());
            println!();
        }
        1 => {
            println!();
            println!("{}", style("─── PROCEDIMIENTO JTAG PARA X300 / X310 ───").yellow().bold());
            println!("  1. Conecta un cable USB estándar al puerto micro-USB frontal etiquetado como 'JTAG'.");
            println!("  2. Enciende el equipo con el interruptor de encendido.");
            println!("  3. Los adaptadores FTDI / Digilent integrados serán detectados automáticamente por el sistema.");
            println!("{}", style("────────────────────────────────────────────").yellow());
            println!();
        }
        2 => {
            println!();
            info("El USRP B200/B210 no utiliza JTAG externo: carga su FPGA dinámicamente mediante el bus USB 3.0.");
            dim("Si no es detectado, prueba otro cable USB 3.0 o conéctalo a un puerto directo de la placa madre.");
            press_enter();
            return;
        }
        _ => return,
    }

    // 3. Enumerate JTAG devices
    step("Detectando cables y dispositivos JTAG conectados...");
    let devices = jtag::enumerate(&djtgcfg);
    let chosen_device = if devices.is_empty() {
        warn("No se detectó ningún programador Digilent JTAG por USB.");
        dim("Verifica que el cable USB JTAG esté conectado y los drivers instalados.");
        println!();
        if let Ok(true) = confirm("¿Deseas ingresar el nombre del dispositivo JTAG manualmente? (ej. JtagHs2)", false) {
            match input("Nombre del dispositivo JTAG:", Some("JtagHs2")) {
                Ok(d) if !d.trim().is_empty() => d.trim().to_string(),
                _ => return,
            }
        } else {
            press_enter();
            return;
        }
    } else {
        ok(&format!("Programador detectado: {}", devices[0].name));
        devices[0].name.clone()
    };

    // 4. Initialize Scan Chain
    step(&format!("Inicializando cadena JTAG en {}...", chosen_device));
    let chain = jtag::init_and_detect(&djtgcfg, &chosen_device);
    if let Some(fpga) = chain {
        ok(&format!("FPGA detectada en la cadena JTAG: {:?}", fpga));
    } else {
        warn("No se pudo identificar la FPGA en la cadena JTAG, pero puedes intentar programarla de todos modos.");
    }

    // 5. Select bitstream image to load
    let local_sets = list_local_images(&config.images_cache_dir);
    let chosen_bitfile = if local_sets.is_empty() {
        warn("No tienes imágenes descargadas en la caché local.");
        dim("  Descarga imágenes primero desde 'Gestionar imágenes FPGA'.");
        match input("Ruta completa al archivo .bit a cargar:", None) {
            Ok(p) => p,
            Err(_) => return,
        }
    } else {
        let mut bits = Vec::new();
        for set in &local_sets {
            for b in &set.bitfiles {
                let fname = b.file_name().unwrap_or_default().to_string_lossy();
                if fname.ends_with(".bit") {
                    bits.push(b.display().to_string());
                }
            }
        }
        if bits.is_empty() {
            match input("Ruta completa al archivo .bit:", None) {
                Ok(p) => p,
                Err(_) => return,
            }
        } else {
            bits.push("📁 Ingresar ruta de archivo manualmente...".to_string());
            bits.push(t!("common.back").to_string());
            let s_idx = match select_str("Selecciona la imagen (.bit) para revivir la FPGA:", &bits) {
                Ok(i) => i,
                Err(_) => return,
            };
            if s_idx == bits.len() - 1 {
                return;
            } else if s_idx == bits.len() - 2 {
                match input("Ruta completa al archivo .bit:", None) {
                    Ok(p) => p,
                    Err(_) => return,
                }
            } else {
                bits[s_idx].clone()
            }
        }
    };

    // 6. Build command and show to user
    let djtg_str = djtgcfg.to_str().unwrap_or("djtgcfg");
    println!();
    println!("{}", style("──────────────────────── RESUMEN DE DESBRICKEO JTAG ────────────────────────").cyan());
    println!("  Dispositivo:  {}", chosen_device);
    println!("  Archivo .bit: {}", chosen_bitfile);
    println!("  Comando CLI que se ejecutará:");
    println!(
        "  {}",
        style(format!("● {} prog -d \"{}\" -i 0 -f \"{}\"", djtg_str, chosen_device, chosen_bitfile))
            .yellow()
            .bold()
    );
    println!("{}", style("────────────────────────────────────────────────────────────────────────────").cyan());
    println!();

    match confirm("¿Deseas programar la FPGA ahora por JTAG?", true) {
        Ok(true) => {
            step("Programando FPGA vía JTAG... No desconectes cables.");
            let res = run_visible(
                djtg_str,
                &["prog", "-d", &chosen_device, "-i", "0", "-f", &chosen_bitfile],
            );
            if res.success {
                ok("¡Programación JTAG completada exitosamente!");
                println!();
                println!(
                    "  {}\n  La FPGA ahora está ejecutando la imagen en memoria RAM volátil. Los LEDs deberían haber encendido.\n",
                    style("ESTADO ACTUAL:").green().bold()
                );
                if let Ok(true) = confirm("¿Deseas proceder ahora a flashearlo permanentemente por Red?", true) {
                    crate::flash::run_flash_network(config);
                    return;
                }
            } else {
                err("La programación JTAG falló.");
                dim("Verifica que el cable esté firmemente conectado y el USRP encendido.");
            }
        }
        _ => {
            info("Operación cancelada.");
        }
    }

    press_enter();
}

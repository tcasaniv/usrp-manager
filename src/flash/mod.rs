use crate::config::AppConfig;
use crate::detection::network::ping;
use crate::executor::run_visible;
use crate::images::local::list_local_images;
use crate::ui::prompts::{confirm, input, select_str};
use crate::ui::styles::{dim, err, info, ok, press_enter, print_section, step, warn};
use console::style;
use rust_i18n::t;
use which::which;

pub fn run_flash_network(config: &AppConfig) {
    print_section(&t!("flash.title"));
    println!(
        "\n  {}\n  Este asistente escribe una imagen FPGA en la memoria flash interna (no volátil) del USRP.\n  A diferencia de JTAG (que se borra al apagar), la imagen guardada se cargará automáticamente cada vez que enciendas el USRP.\n",
        style("● ¿En qué consiste el Flasheo Permanente?").cyan().bold()
    );
    warn("PRECAUCIÓN: NO desconectes el cable Ethernet ni apagues el USRP durante el flasheo.");
    println!();

    // 1. Check uhd_image_loader in PATH
    let uhd_loader = match which("uhd_image_loader") {
        Ok(p) => p,
        Err(_) => {
            err(&t!("flash.no_uhd"));
            dim("  Instala o activa UHD con 'Gestionar drivers UHD' antes de continuar.");
            press_enter();
            return;
        }
    };

    // 2. Select Connection Mode
    let mode_options = vec![
        "🔍 Autodetectar USRP en la red (uhd_find_devices)".to_string(),
        "⌨️  Ingresar dirección IP manualmente (ej. 192.168.10.2)".to_string(),
        t!("common.back").to_string(),
    ];
    let mode_idx = match select_str("¿Cómo deseas conectar con el USRP?", &mode_options) {
        Ok(i) => i,
        Err(_) => return,
    };

    let target_ip = match mode_idx {
        0 => {
            step("Escaneando red con uhd_find_devices...");
            let find_bin = which("uhd_find_devices").unwrap_or_else(|_| std::path::PathBuf::from("uhd_find_devices"));
            let find_res = run_visible(find_bin.to_str().unwrap_or("uhd_find_devices"), &[]);
            let mut detected_ips = Vec::new();
            for line in find_res.output.lines() {
                let l = line.trim();
                if l.starts_with("addr:") {
                    if let Some((_, ip_val)) = l.split_once(':') {
                        let ip_str = ip_val.trim();
                        if !ip_str.is_empty() && !detected_ips.contains(&ip_str.to_string()) {
                            detected_ips.push(ip_str.to_string());
                        }
                    }
                }
            }
            if detected_ips.is_empty() {
                warn("No se detectó ningún USRP automáticamente.");
                dim("Verifica que el cable esté en el puerto SFP0 y tu IP local sea 192.168.10.1.");
                println!();
                match input(&t!("flash.enter_ip"), Some("192.168.10.2")) {
                    Ok(s) => s,
                    Err(_) => return,
                }
            } else {
                detected_ips.push(t!("common.back").to_string());
                match select_str("Selecciona el USRP detectado:", &detected_ips) {
                    Ok(idx) if idx < detected_ips.len() - 1 => detected_ips[idx].clone(),
                    _ => return,
                }
            }
        }
        1 => match input(&t!("flash.enter_ip"), Some("192.168.10.2")) {
            Ok(s) => s,
            Err(_) => return,
        },
        _ => return,
    };

    // 3. Test ping
    step(&format!("Probando conectividad (ping) con {}...", target_ip));
    if ping(&target_ip) {
        ok(&format!("USRP responde en {}", target_ip));
    } else {
        warn(&format!("El USRP en {} no respondió al ping.", target_ip));
        dim("  Si el USRP está brickeado (sin LEDs frontales), primero debes usar 'Desbrickear por JTAG'.");
        if let Ok(false) = confirm("¿Deseas intentar continuar de todas formas?", false) {
            press_enter();
            return;
        }
    }

    // 4. Select USRP Model with friendly guidance
    println!();
    println!(
        "  {}\n  Cada modelo utiliza un formato y tipo de imagen específico:",
        style("● Selección de Modelo de USRP").cyan().bold()
    );
    let models = vec![
        "USRP N200 / N210  (Imágenes en formato .bin)".to_string(),
        "USRP X300 / X310  (Imágenes en formato .bit, variantes HG o XG)".to_string(),
        "USRP N300 / N310 / N320 (Arquitectura HG/XG/AA/WX)".to_string(),
        "USRP E310 / E320".to_string(),
        t!("common.back").to_string(),
    ];
    let model_idx = match select_str("Selecciona el modelo de tu USRP:", &models) {
        Ok(i) => i,
        Err(_) => return,
    };
    if model_idx >= models.len() - 1 {
        return;
    }

    let usrp_type = match model_idx {
        0 => "n200",
        1 => "x300",
        2 => "n300",
        3 => "e310",
        _ => "usrp",
    };

    // 5. Select Image Version and Variant with explanation
    println!();
    println!(
        "  {}\n  • HG (1G/10G): Puerto 0 a 1 Gigabit (estándar común RJ45), Puerto 1 a 10G. (Recomendado)\n  • XG (Dual 10G): Ambos puertos SFP+ a 10 Gigabit (requiere tarjeta 10G en PC).\n  • Sin sufijo / Default: Configuración estándar del fabricante para N210.",
        style("● ¿Qué significan las variantes de imagen?").cyan().bold()
    );

    let local_sets = list_local_images(&config.images_cache_dir);
    let chosen_fpga_path = if local_sets.is_empty() {
        warn("No tienes imágenes descargadas en la caché local.");
        dim("  Puedes descargarlas en 'Gestionar imágenes FPGA'.");
        println!();
        match input("Ruta completa al archivo de imagen (.bit o .bin):", None) {
            Ok(p) => p,
            Err(_) => return,
        }
    } else {
        let mut img_choices = Vec::new();
        for set in &local_sets {
            for bit in &set.bitfiles {
                let fname = bit.file_name().unwrap_or_default().to_string_lossy();
                // Filter matching files for selected model if possible
                if (usrp_type == "n200" && (fname.contains("n210") || fname.contains("n200")))
                    || (usrp_type == "x300" && fname.contains("x3"))
                    || (usrp_type == "n300" && fname.contains("n3"))
                    || usrp_type == "e310"
                    || model_idx > 1
                {
                    img_choices.push(bit.display().to_string());
                }
            }
        }

        // If no model-specific files found, offer all available
        if img_choices.is_empty() {
            for set in &local_sets {
                for bit in &set.bitfiles {
                    img_choices.push(bit.display().to_string());
                }
            }
        }

        img_choices.push("📁 Ingresar ruta de archivo manualmente...".to_string());
        img_choices.push(t!("common.back").to_string());

        let sel_idx = match select_str("Selecciona la imagen FPGA que deseas grabar:", &img_choices) {
            Ok(i) => i,
            Err(_) => return,
        };

        if sel_idx == img_choices.len() - 1 {
            return;
        } else if sel_idx == img_choices.len() - 2 {
            match input("Ruta completa al archivo de imagen:", None) {
                Ok(p) => p,
                Err(_) => return,
            }
        } else {
            img_choices[sel_idx].clone()
        }
    };

    // 6. Build Command & Confirm
    let loader_str = uhd_loader.to_str().unwrap_or("uhd_image_loader");
    let args_spec = format!("type={},addr={}", usrp_type, target_ip);

    println!();
    println!("{}", style("────────────────────────── RESUMEN DE FLASHEO ──────────────────────────").cyan());
    println!("  Dispositivo:  USRP {} ({})", usrp_type.to_uppercase(), target_ip);
    println!("  Archivo FPGA: {}", chosen_fpga_path);
    println!("  Comando CLI que se ejecutará:");
    println!(
        "  {}",
        style(format!("● {} --args \"{}\" --fpga-path \"{}\"", loader_str, args_spec, chosen_fpga_path))
            .yellow()
            .bold()
    );
    println!("{}", style("────────────────────────────────────────────────────────────────────────").cyan());
    println!();

    match confirm("¿Estás seguro de que deseas flashear esta imagen permanentemente?", true) {
        Ok(true) => {
            step("Ejecutando flasheo. No apagues el equipo...");
            let res = run_visible(
                loader_str,
                &["--args", &args_spec, "--fpga-path", &chosen_fpga_path],
            );
            if res.success {
                ok("¡Flasheo completado con éxito!");
                println!();
                println!(
                    "  {}\n  Apaga el USRP, espera 5 segundos y vuelve a encenderlo para cargar la nueva imagen.",
                    style("IMPORTANTE: Reinicio requerido").green().bold()
                );
            } else {
                err("El proceso de flasheo devolvió un error.");
                dim("Verifica que la versión de UHD activa sea compatible con el USRP.");
            }
        }
        _ => {
            info("Operación cancelada.");
        }
    }

    press_enter();
}

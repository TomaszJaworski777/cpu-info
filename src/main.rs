use std::io::Write;

use chrono::Local;

use crate::{
    cpu_info::CpuInfo,
    utils::{AlignString, Colors, bytes_to_string, clear_terminal_screen, time_to_string},
};

mod cpu_info;
mod utils;

fn main() {
    let cpu_info = if cfg!(target_os = "windows") {
        use crate::cpu_info::WindowsCpuInfo;
        WindowsCpuInfo::new()
    } else if cfg!(target_os = "linux") {
        use crate::cpu_info::WindowsCpuInfo;
        WindowsCpuInfo::new() //TODO: Change to linux
    } else {
        panic!("Target OS not supported!")
    };

    let mut last_width = 0;

    print!("\x1B[?25l");
    let _ = std::io::stdout().flush();

    loop {
        print_data(&cpu_info, &mut last_width);
    }
}

fn print_data<Cpu>(info: &Cpu, last_width: &mut usize)
where
    Cpu: CpuInfo,
{
    let (width, _) = term_size::dimensions().unwrap_or((80, 0));

    if width != *last_width {
        clear_terminal_screen();
        *last_width = width;
    }

    _ = term_cursor::set_pos(0, 0);

    let side_panel_width = (((width - 40) / 2) - 2).max(25);
    let mut flag_lines = vec![String::new(); 13];
    let mut idx = 0;
    for flag in info.static_data().flags() {
        flag_lines[idx].push_str(format!("{}, ", flag).as_str());

        if flag_lines[idx].len() >= side_panel_width - 10 {
            idx += 1;
        }
    }
    println!(
        "\n{}",
        format!(
            " ┌{}┐ {} ┌{}┐ ",
            "─".repeat(side_panel_width - 2),
            info.static_data().brand().align_to_center(40).white(),
            "─".repeat(side_panel_width - 2)
        )
        .gray()
    );
    println!(
        "{}",
        format!(
            " │{}│ {} │ {}│ ",
            format!(" Vendor:   {}", info.static_data().vendor())
                .align_to_left(side_panel_width - 2)
                .white(),
            " ".align_to_center(40),
            flag_lines[0].align_to_left(side_panel_width - 3).white()
        )
        .gray()
    );
    println!(
        "{}",
        format!(
            " │{}│ {} │ {}│ ",
            format!(" Arch:     {}", info.static_data().architecture())
                .align_to_left(side_panel_width - 2)
                .white(),
            " ".align_to_center(40),
            flag_lines[1].align_to_left(side_panel_width - 3).white()
        )
        .gray()
    );
    println!(
        "{}",
        format!(
            " │{}│ {} │ {}│ ",
            format!(" Family:   {}", info.static_data().family())
                .align_to_left(side_panel_width - 2)
                .white(),
            format!(
                "{}{}{}",
                " ".repeat(10),
                "┏━━━━━━━━━━━━━━━━━┓",
                " ".repeat(11)
            )
            .align_to_center(40),
            flag_lines[2].align_to_left(side_panel_width - 3).white()
        )
        .gray()
    );
    println!(
        "{}",
        format!(
            " │{}│ {} │ {}│ ",
            format!(" Model:    {}", info.static_data().model())
                .align_to_left(side_panel_width - 2)
                .white(),
            format!(
                "{}{}{}",
                " ".repeat(10),
                format!(
                    "┃{}┃",
                    format!(
                        "{}c/{}t",
                        info.static_data().cores(),
                        info.static_data().threads()
                    )
                    .align_to_center(17)
                    .white()
                ),
                " ".repeat(11)
            )
            .align_to_center(40),
            flag_lines[3].align_to_left(side_panel_width - 3).white()
        )
        .gray()
    );
    println!(
        "{}",
        format!(
            " │{}│ {} │ {}│ ",
            format!(" Stepping: {}", info.static_data().stepping())
                .align_to_left(side_panel_width - 2)
                .white(),
            format!(
                "{}{}{}",
                " ".repeat(10),
                "┃                 ┃",
                " ".repeat(11)
            )
            .align_to_center(40),
            flag_lines[4].align_to_left(side_panel_width - 3).white()
        )
        .gray()
    );
    println!(
        "{}",
        format!(
            " │{}│ {} │ {}│ ",
            " ".align_to_center(side_panel_width - 2),
            format!(
                "{}{}{}",
                " ".repeat(10),
                format!(
                    "┃{}┃",
                    format!(
                        " L1: {}B",
                        bytes_to_string(info.static_data().l1_cache() as u128)
                    )
                    .align_to_left(17)
                    .white()
                ),
                " ".repeat(11)
            )
            .align_to_center(40),
            flag_lines[5].align_to_left(side_panel_width - 3).white()
        )
        .gray()
    );
    println!(
        "{}",
        format!(
            " │{}│ {} │ {}│ ",
            format!(
                " Base clock speed:  {}MHz",
                info.static_data().clock_speed()
            )
            .align_to_left(side_panel_width - 2)
            .white(),
            format!(
                "{}{}{}",
                " ".repeat(10),
                format!(
                    "┃{}┃",
                    format!(
                        " L2: {}B",
                        bytes_to_string(info.static_data().l2_cache() as u128)
                    )
                    .align_to_left(17)
                    .white()
                ),
                " ".repeat(11)
            )
            .align_to_center(40),
            flag_lines[6].align_to_left(side_panel_width - 3).white()
        )
        .gray()
    );
    println!(
        "{}",
        format!(
            " │{}│ {} │ {}│ ",
            format!(
                " Tubdo clock speed: {}MHz",
                info.static_data().clock_speed_turbo()
            )
            .align_to_left(side_panel_width - 2)
            .white(),
            format!(
                "{}{}{}",
                " ".repeat(10),
                format!(
                    "┃{}┃",
                    format!(
                        " L3: {}B",
                        bytes_to_string(info.static_data().l3_cache() as u128)
                    )
                    .align_to_left(17)
                    .white()
                ),
                " ".repeat(11)
            )
            .align_to_center(40),
            flag_lines[7].align_to_left(side_panel_width - 3).white()
        )
        .gray()
    );
    println!(
        "{}",
        format!(
            " │{}│ {} │ {}│ ",
            " ".align_to_center(side_panel_width - 2),
            format!(
                "{}{}{}",
                " ".repeat(10),
                "┃                 ┃",
                " ".repeat(11)
            )
            .align_to_center(40),
            flag_lines[8].align_to_left(side_panel_width - 3).white()
        )
        .gray()
    );
    println!(
        "{}",
        format!(
            " │{}│ {} │ {}│ ",
            " ".align_to_center(side_panel_width - 2),
            format!(
                "{}{}{}",
                " ".repeat(10),
                "┃                 ┃",
                " ".repeat(11)
            )
            .align_to_center(40),
            flag_lines[9].align_to_left(side_panel_width - 3).white()
        )
        .gray()
    );
    println!(
        "{}",
        format!(
            " │{}│ {} │ {}│ ",
            " ".align_to_center(side_panel_width - 2),
            format!(
                "{}{}{}",
                " ".repeat(10),
                format!(
                    "┃{}┃",
                    format!("{}", time_to_string(info.uptime() as u128 * 1000))
                        .align_to_center(17)
                        .white()
                ),
                " ".repeat(11)
            )
            .align_to_center(40),
            flag_lines[10].align_to_left(side_panel_width - 3).white()
        )
        .gray()
    );
    println!(
        "{}",
        format!(
            " │{}│ {} │ {}│ ",
            " ".align_to_center(side_panel_width - 2),
            format!(
                "{}{}{}",
                " ".repeat(10),
                "┗━━━━━━━━━━━━━━━━━┛",
                " ".repeat(11)
            )
            .align_to_center(40),
            flag_lines[11].align_to_left(side_panel_width - 3).white()
        )
        .gray()
    );
    println!(
        "{}",
        format!(
            " │{}│ {} │ {}│ ",
            format!(" Microcode:  {}", info.static_data().microcode_version())
                .align_to_left(side_panel_width - 2)
                .white(),
            " ".align_to_center(40),
            flag_lines[12].align_to_left(side_panel_width - 3).white()
        )
        .gray()
    );
    println!(
        "{}",
        format!(
            " └{}┘ {} └{}┘ \n",
            "─".repeat(side_panel_width - 2),
            format!("{} ", Local::now().format("%H:%M:%S"))
                .align_to_center(40)
                .white(),
            "─".repeat(side_panel_width - 2)
        )
        .gray()
    );
}

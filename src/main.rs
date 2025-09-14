use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, EnableMouseCapture, MouseEventKind},
    execute,
    style::{Color, Print, SetForegroundColor},
    terminal::{self, disable_raw_mode, enable_raw_mode, ClearType},
};

use std::io::{self};
use std::time::{Duration, Instant};

// screen size
const W: usize = 80; 
const H: usize = 25;

#[derive(Clone)]
struct Enemy {
    path: usize,
    hp: i32,
    living: bool,
}

#[derive(Clone)]
struct Tower {
    x: usize,
    y: usize,
    last_shot: Instant,
    direction: i32, // 0=up, 1=right, 2=down, 3=left
}

#[derive(Clone)]
struct Projectile {
    x: usize,
    y: usize,
    alive: bool,
    direction: i32,
}


// main menu 
fn show_menu(stdout: &mut io::Stdout) -> i32 { 
    execute!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0)).unwrap();
    
    println!("=== DAVID's TOWER DEFENSE ===");
    println!();
    println!("Choose difficulty:");
    println!("1. Easy   (5 enemies, 100 gold)");
    println!("2. Medium (10 enemies, 30 gold)");
    println!("3. Hard   (15 enemies, 50 gold)");
    println!();
    println!("Press 1, 2, or 3 to select:");

    // for selecting difficulty
    loop {
        if event::poll(Duration::from_millis(50)).unwrap() {
            if let Event::Key(KeyEvent { code, .. }) = event::read().unwrap() {
                match code {
                    KeyCode::Char('1') => return 1,
                    KeyCode::Char('2') => return 2,
                    KeyCode::Char('3') => return 3,
                    _ => {}
                }
            }
        }
    }
}

// for creating paths
fn create_path(difficulty: i32) -> Vec<(usize, usize)> {
    let mut path = Vec::new();
    
    if difficulty == 1 { // Easy
        for x in 1..25 {
            path.push((x, 8));
        }
        for y in 9..15 {
            path.push((24, y));
        }
        for x in 25..55 {
            path.push((x, 14));
        }
        for y in 15..21 {
            path.push((54, y));
        }
        for x in 55..79 {
            path.push((x, 20));
        }
    } else if difficulty == 2 { // Medium
        for y in 1..15 {
            path.push((40, y));
        }
        for x in 40..79 {
            path.push((x, 15));
        }
    } else { // Hard (3)
        for x in 1..79 {
            path.push((x, 12));
        }
    }
    
    path
}

// for checking if tower is on path
fn is_on_path(x: usize, y: usize, path: &Vec<(usize, usize)>) -> bool {
    for i in 0..path.len() {
        let (px, py) = path[i];
        if x == px && y == py {
            return true;
        }
    }
    false
}

// for updating every frame
fn update(
    enemies: &mut Vec<Enemy>,
    towers: &mut Vec<Tower>,
    projectiles: &mut Vec<Projectile>,
    path: &Vec<(usize, usize)>,
    total_enemies: usize,
    spawned: &mut usize,
    last_spawn: &mut Instant,
    wave_started: bool,
    gold: &mut i32,
    lives: &mut i32,
) {
    // wave start
    if wave_started && *spawned < total_enemies && last_spawn.elapsed() >= Duration::from_millis(1500) {
        enemies.push(Enemy {
            path: 0,
            hp: 1,
            living: true,
        });
        *spawned += 1;
        *last_spawn = Instant::now();
    }

    // enemy movement system
    for i in 0..enemies.len() {
        if enemies[i].living {
            if enemies[i].path + 1 < path.len() {
                enemies[i].path += 1;
            } else {
                *lives -= 1;
                enemies[i].living = false;
            }
        }
    }

    // tower shooting mechanic
    for i in 0..towers.len() {
        if towers[i].last_shot.elapsed() >= Duration::from_millis(2000) {
            let mut new_x = towers[i].x;
            let mut new_y = towers[i].y;
            
            // tower direction for shooting
            if towers[i].direction == 0 && towers[i].y > 1 { // up
                new_y = towers[i].y - 1;
            } else if towers[i].direction == 1 && towers[i].x < W - 2 { // right
                new_x = towers[i].x + 1;
            } else if towers[i].direction == 2 && towers[i].y < H - 2 { // down
                new_y = towers[i].y + 1;
            } else if towers[i].direction == 3 && towers[i].x > 1 { // left
                new_x = towers[i].x - 1;
            }
            
            // tower projectile checking system
            if new_x != towers[i].x || new_y != towers[i].y {
                projectiles.push(Projectile {
                    x: new_x,
                    y: new_y,
                    alive: true,
                    direction: towers[i].direction,
                });
                towers[i].last_shot = Instant::now();
            }
        }
    }

    // projectile system
    for i in 0..projectiles.len() {
        if projectiles[i].alive {
            let mut moved = false;
            
            if projectiles[i].direction == 0 && projectiles[i].y > 1 { // up
                projectiles[i].y -= 1;
                moved = true;
            } else if projectiles[i].direction == 1 && projectiles[i].x < W - 2 { // right
                projectiles[i].x += 1;
                moved = true;
            } else if projectiles[i].direction == 2 && projectiles[i].y < H - 2 { // down
                projectiles[i].y += 1;
                moved = true;
            } else if projectiles[i].direction == 3 && projectiles[i].x > 1 { // left
                projectiles[i].x -= 1;
                moved = true;
            }
            
            if !moved {
                projectiles[i].alive = false;
            }
        }
    }

    // collision detection 
    for i in 0..projectiles.len() {
        if projectiles[i].alive {
            for j in 0..enemies.len() {
                if enemies[j].living {
                    let (ex, ey) = path[enemies[j].path];
                    if projectiles[i].x == ex && projectiles[i].y == ey {
                        enemies[j].hp -= 1;
                        projectiles[i].alive = false;
                        if enemies[j].hp <= 0 {
                            enemies[j].living = false;
                            *gold += 2; // gold reward
                        }
                    }
                }
            }
        }
    }

    // to remove dead projectiles
    let mut new_projectiles = Vec::new();
    for i in 0..projectiles.len() {
        if projectiles[i].alive {
            new_projectiles.push(projectiles[i].clone());
        }
    }
    projectiles.clear();
    for projectile in new_projectiles {
        projectiles.push(projectile);
    }
}

// for terminal rendering
fn render(
    stdout: &mut io::Stdout,
    screen: &mut Vec<Vec<char>>,
    enemies: &Vec<Enemy>,
    towers: &Vec<Tower>,
    projectiles: &Vec<Projectile>,
    path: &Vec<(usize, usize)>,
    spawned: usize,
    total_enemies: usize,
    gold: i32,
    wave_started: bool,
    wave_number: i32,
    lives: &mut i32
) {
    // clears screen 
    for y in 0..H {
        for x in 0..W {
            screen[y][x] = ' ';
        }
    }

    // border
    for y in 0..H {
        for x in 0..W {
            if y == 0 || y == H - 1 || x == 0 || x == W - 1 {
                screen[y][x] = '#';
            }
        }
    }

    // path 
    for i in 0..path.len() {
        let (px, py) = path[i];
        if px < W && py < H {
            screen[py][px] = '.';
        }
    }

    // enemies
    for i in 0..enemies.len() {
        if enemies[i].living {
            let (ex, ey) = path[enemies[i].path];
            if ex < W && ey < H {
                screen[ey][ex] = '@';
            }
        }
    }

    // towers
    for i in 0..towers.len() {
        if towers[i].x < W && towers[i].y < H {
            let symbol = if towers[i].direction == 0 { '^' }
                        else if towers[i].direction == 1 { '>' }
                        else if towers[i].direction == 2 { 'v' }
                        else { '<' };
            screen[towers[i].y][towers[i].x] = symbol;
        }
    }

    // projectiles
    for i in 0..projectiles.len() {
        if projectiles[i].alive {
            if projectiles[i].x < W && projectiles[i].y < H {
                screen[projectiles[i].y][projectiles[i].x] = '*';
            }
        }
    }

    // renders the previous elements to le terminal
    execute!(stdout, cursor::MoveTo(0, 0)).unwrap();
    for y in 0..H {
        for x in 0..W {
            let cell = screen[y][x];
            match cell {
                '#' => {
                    execute!(stdout, SetForegroundColor(Color::Green), Print('#')).unwrap();
                }
                '.' => {
                    execute!(stdout, SetForegroundColor(Color::DarkYellow), Print('.')).unwrap();
                }
                '@' => {
                    execute!(stdout, SetForegroundColor(Color::Red), Print('@')).unwrap();
                }
                '^' | '>' | 'v' | '<' => {
                    execute!(stdout, SetForegroundColor(Color::Blue), Print(cell)).unwrap();
                }
                '*' => {
                    execute!(stdout, SetForegroundColor(Color::Cyan), Print('*')).unwrap();
                }
                _ => {
                    execute!(stdout, SetForegroundColor(Color::White), Print(cell)).unwrap();
                }
            }
        }
        println!();
    }

    // counts how many enemies are alive
    let mut alive_count = 0;
    for i in 0..enemies.len() {
        if enemies[i].living {
            alive_count += 1;
        }
    }

    // HUD
    execute!(stdout, SetForegroundColor(Color::White)).unwrap();
    println!("Wave {} | Enemies: {}/{} | Alive: {} | Gold: {} | Towers: {} | Lives: {}            ", 
             wave_number, spawned, total_enemies, alive_count, gold, towers.len(), lives);
    
    if wave_started == false {
        println!("Press ENTER to start wave | Click to place towers (10 gold) | Click towers to rotate");
    } else {
        println!("Click to place towers (10 gold) | Click towers to rotate | Press 'q' to quit        ");
    }
}

fn main() {
    let mut stdout = io::stdout();
    
    enable_raw_mode().unwrap();
    execute!(stdout, terminal::Clear(ClearType::All), cursor::Hide, cursor::MoveTo(0, 0), EnableMouseCapture).unwrap();

    // show menu func call to receive difficulty from user
    let difficulty = show_menu(&mut stdout);
    
    // difficulty settings
    let (mut total_enemies, starting_gold) = if difficulty == 1 {
        (5, 100)  // easy
    } else if difficulty == 2 {
        (10, 30)  // medium  
    } else {
        (15, 50)  // hard
    };
    
    // player variables
    let path = create_path(difficulty);
    let mut wave_number = 1;
    let mut running = true;
    let mut towers:Vec<Tower> = Vec::new(); 
    let mut gold = starting_gold;
    let mut lives = 10;
    let mut waves = 0;

    while running {
        // game variables
        let mut last_spawn = Instant::now();
        let mut spawned = 0;
        let mut enemies = Vec::new();
        let mut projectiles = Vec::new();
        let mut wave_started = false;
        let mut wave_complete = false;

        let fps = Duration::from_millis(200); // fps duh
        let mut last_frame = Instant::now(); 
        let mut screen = vec![vec![' '; W]; H]; // screen

        // game loop 
        while running && wave_complete == false { 
            // input 
            if event::poll(Duration::from_millis(1)).unwrap() {
                match event::read().unwrap() {
                    Event::Key(KeyEvent { code, .. }) => {
                        match code {
                            KeyCode::Char('q') => { // quit the game
                                running = false;
                            }
                            KeyCode::Enter => { // start the wvae
                                if wave_started == false {
                                    wave_started = true;
                                }
                            }
                            _ => {}
                        }
                    }
                    Event::Mouse(m) => { // tower mouse input
                        if let MouseEventKind::Down(_) = m.kind {
                            let x = m.column as usize;
                            let y = m.row as usize;
                            
                            if x > 0 && x < W - 1 && y > 0 && y < H - 2 { // to rotate tower
                                let mut clicked_tower = false;
                                for i in 0..towers.len() {
                                    if towers[i].x == x && towers[i].y == y {
                                        // Rotate tower direction
                                        towers[i].direction = (towers[i].direction + 1) % 4;
                                        clicked_tower = true;
                                        break;
                                    }
                                }

                                if !clicked_tower && !is_on_path(x, y, &path) && gold >= 10 { // to place tower on an empty cell 
                                    towers.push(Tower {
                                        x,
                                        y,
                                        last_shot: Instant::now(),
                                        direction: 3, // Start facing left
                                    });
                                    gold -= 10;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            // update game
            if last_frame.elapsed() >= fps {
                last_frame = Instant::now();

                if wave_started {
                    update(
                        &mut enemies,
                        &mut towers,
                        &mut projectiles,
                        &path,
                        total_enemies,
                        &mut spawned,
                        &mut last_spawn,
                        wave_started,
                        &mut gold,
                        &mut lives,
                    );

                    // win-lose condition 
                    if lives == 0 {
                        running = false;
                    }

                    if waves == 3 {
                        running = false;
                    }
                    // Check if wave is complete
                    if spawned == total_enemies {
                        let mut all_dead = true;
                        for i in 0..enemies.len() {
                            if enemies[i].living {
                                all_dead = false;
                                break;
                            }
                        }
                        if all_dead {
                            wave_complete = true;
                            waves += 1;
                            gold += 25; // Bonus gold for completing wave
                        }
                    }
                }

                render(
                    &mut stdout,
                    &mut screen,
                    &enemies,
                    &towers,
                    &projectiles,
                    &path,
                    spawned,
                    total_enemies,
                    gold,
                    wave_started,
                    wave_number,
                    &mut lives,
                );
            }
        }

        // when wave is complete
        if wave_complete && running {
            execute!(stdout, cursor::MoveTo(0, 28), SetForegroundColor(Color::Green)).unwrap();
            println!("WAVE {} COMPLETE! +25 Gold!", wave_number);
            println!("Press ENTER for next wave or 'q' to quit");
            
            let mut waiting = true;
            while waiting && running {
                if event::poll(Duration::from_millis(50)).unwrap() {
                    if let Event::Key(KeyEvent { code, .. }) = event::read().unwrap() {
                        match code {
                            KeyCode::Enter => { // continuing
                                wave_number += 1;
                                total_enemies += 5;
                                execute!(stdout, cursor::MoveTo(0,28), terminal::Clear(ClearType::UntilNewLine)).unwrap();
                                waiting = false;
                            }
                            KeyCode::Char('q') => { // exiting
                                running = false;
                                waiting = false;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    // game over screen 
    while lives == 0 {
       execute!(stdout, cursor::MoveTo(0, 26), SetForegroundColor(Color::Red)).unwrap();
       println!("================================GAME OVER===================================");
       println!("=============================press 'q' to exit==============================");
       if event::poll(Duration::from_millis(50)).unwrap() {
           if let Event::Key(KeyEvent { code, .. }) = event::read().unwrap() {
           match code {
               KeyCode::Char('q') => { lives += 1; }
               _ => {}

               }
           }
       }
    }

    while waves == 3 {
       execute!(stdout, cursor::MoveTo(0, 26), SetForegroundColor(Color::Yellow)).unwrap();
       println!("================================YOU WIN!!===================================");
       println!("=============================press 'q' to exit==============================");
       if event::poll(Duration::from_millis(50)).unwrap() {
           if let Event::Key(KeyEvent { code, .. }) = event::read().unwrap() {
           match code {
               KeyCode::Char('q') => { waves += 1; }
               _ => {}

               }
           }
       }
    }
    
    
    execute!(stdout, cursor::MoveTo(0, 26), terminal::Clear(ClearType::All) ,cursor::Show).unwrap();
    println!("Quiting......");
    disable_raw_mode().unwrap();
}
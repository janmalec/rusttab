use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

extern crate plotters;
use plotters::prelude::*;

struct PlotDimensions {
    xlabel: String,
    ylabel: String,
    xlo: f64,
    xhi: f64,
    ylo: f64,
    yhi: f64,
    xdim_lo: f64,
    xdim_hi: f64,
    ydim_lo: f64,
    ydim_hi: f64,
    no_curves: i8,
    no_points: i8,
    line_thickness: f64,
    autoscale_x: bool,
    autoscale_y: bool,
    add_border: bool,
    xy_grid: i8,
    ratio_first: bool,
    title1: String,
    title2: String
}

impl Default for PlotDimensions {
    fn default() -> PlotDimensions {
        PlotDimensions {
            xlabel: String::from(""),
            ylabel: String::from(""),
            xlo: f64::INFINITY,
            xhi: -1.,
            ylo: f64::INFINITY,
            yhi: -1.,
            xdim_lo: 0.,
            xdim_hi: 13.5,
            ydim_lo: 0.,
            ydim_hi: 10.,
            no_curves: 1,
            no_points: 0,
            line_thickness: 1.5,
            autoscale_x: true,
            autoscale_y: true,
            add_border: false,
            xy_grid: 0,
            ratio_first: false,
            title1: String::from(""),
            title2: String::from("")
        }
    }
}

enum ReadingState {
    Head1,
    Head2,
    Xlabel,
    Ylabel,
    Title1,
    Title2,
    Line1,
    Line2,
    Empty
}


fn main() {
    // let colors: [&dyn Color; 3] = [&RED, &BLUE, &BLACK];
    // Check if a file is provided as a command line argument
    // If not, use the default file
    let args: Vec<String> = std::env::args().collect();
    let file_name = if args.len() > 1 {
        &args[1]
    } else {
        "test_files/ISOPLT.P92"
    };
    // File hosts must exist in current path before this produces output
    if let Ok(lines) = read_lines(file_name) {
        // if a second comand line argument is provided, that is the cur file, otherwise
        // the CUR file name is the same as the P92 file name except .CUR extension
        let cur_filename = if args.len() > 2 {
            args[2].clone()
        } else {
            let mut cur_filename = String::from(file_name);
            cur_filename.replace_range(cur_filename.len()-3.., "CUR");
            cur_filename
        };
        if let Ok(mut curves) = read_lines(&cur_filename) {
            // Consumes the iterator, returns an (Optional) String
            let mut settings: PlotDimensions = PlotDimensions{..Default::default()};
            let mut state = ReadingState::Head1;
            let mut labels : Vec<String> = Vec::new();
            lines.enumerate().for_each(|(_i, line)| {
                if let Ok(to_parse) = line {
                    //println!("{}", to_parse);
                    if to_parse.trim().is_empty() {
                        settings = PlotDimensions{..Default::default()};
                        state = ReadingState::Empty;
                        labels.clear();
                    }
                    //println!("i: {} l:{}", i, to_parse);
                    parse(to_parse, &mut settings, &mut state);
                    // create an output directory with derived name from cur_filename without ext
                    let dir_name: &str = cur_filename.split(".").next().unwrap();
                    if !Path::new(&dir_name).exists() {
                        std::fs::create_dir(dir_name).unwrap();
                    }

                    if  matches!(state, ReadingState::Line2) {
                        //let file_name = String::from("out/") + &settings.title1.clone() + ".png";
                        let file_name = String::from(dir_name) + "/" + &settings.title1.clone() + ".svg";
                        //let root = BitMapBackend::new(&file_name, (1024, 768)).into_drawing_area();
                        let root = SVGBackend::new(&file_name, (1024, 768)).into_drawing_area();
                        let (upper, lower) = root.split_vertically((768.*0.75) as i32);
                        //data.clear();
                        let mut data : Vec<Vec<(f64, f64)>> = Vec::new();
                        for j in 0..settings.no_curves{
                            let mut read_data : Vec<(f64, f64)> = Vec::new();
                            let title =  curves.next().unwrap().unwrap();
                            println!("j: {} Title: {}", j, title);
                            while let Some(entry) = curves.next() {
                                if let Ok(point) = entry{
                                    if point.trim().is_empty(){
                                        root.fill(&WHITE).unwrap();
                                        break;
                                    }
                                    //println!("{}", point);
                                    let (x, y) = parse_point(point);
                                    if settings.autoscale_x {
                                        settings.xlo = f64::min(settings.xlo, x);
                                        settings.xhi = f64::max(settings.xhi, x);
                                    }
                                    if settings.autoscale_y {
                                        settings.ylo = f64::min(settings.ylo, y);
                                        settings.yhi = f64::max(settings.yhi, y);
                                    }
                                    read_data.push((x, y));
                                }
                            }
                            
                            data.push(read_data);
                            labels.push(title);
                        }
                        upper.draw(&Text::new(settings.title1.clone(), (20, 20), ("arial", 30))).unwrap();
                        upper.draw(&Text::new(settings.title2.clone(), (20, 40), ("arial", 30))).unwrap();
                        //let full_title = settings.title1.clone();
                        //full_title += "\n";
                        //full_title += &settings.title2;
                        let mut chart = ChartBuilder::on(&upper)
                        //.caption(full_title, ("arial", 28))
                        .margin_left(40)
                        .margin_right(40)
                        .margin_top(80)
                        .margin_bottom(0)
                        .set_label_area_size(LabelAreaPosition::Left, 90)
                        //.set_label_area_size(LabelAreaPosition::Bottom, 60)
                        .build_cartesian_2d(settings.xlo..settings.xhi, settings.ylo..settings.yhi).unwrap();
                        
                        chart.configure_mesh()
                        //.disable_mesh()
                        //.disable_x_axis()
                        .x_desc(&settings.xlabel)
                        .y_desc(&settings.ylabel)
                        .axis_desc_style(("arial", 28))
                        .x_label_formatter(&|x| format!("{:.1e}", x))
                        .y_label_formatter(&|x| format!("{:.1e}", x))
                        .label_style(("arial", 24)).draw().unwrap();
                        
                        for idx in 0..data.len(){
                            let d = data[idx].clone();
                            let color = Palette99::pick(idx);
                            chart
                            .draw_series(LineSeries::new(d, Palette99::pick(idx).filled().stroke_width(3)))
                            .unwrap()
                            .label(&labels[idx])
                            .legend(move |(x, y)| {
                                Rectangle::new([(x - 5, y - 5), (x + 5, y + 5)], color.filled())});
                            }
                        chart
                            .configure_series_labels()
                            .background_style(&WHITE.mix(0.8))
                            .border_style(&WHITE)
                            .label_font(("arial", 24))
                            .draw().unwrap();
                            
                        let mut low_chart = ChartBuilder::on(&lower)
                            .margin_left(40)
                            .margin_right(40)
                            .set_label_area_size(LabelAreaPosition::Left, 90)
                            .set_label_area_size(LabelAreaPosition::Bottom, 60)
                            .build_cartesian_2d(settings.xlo..settings.xhi, -100.0..100.0).unwrap();
                            
                        low_chart.configure_mesh()
                            .y_labels(5)
                            .x_desc(&settings.xlabel)
                            .y_desc(String::from("Ratio"))
                            .axis_desc_style(("arial", 28))
                            .x_label_formatter(&|x| format!("{:.1e}", x))
                            //.y_label_formatter(&|x| format!("{:.1e}", x))
                            .label_style(("arial", 24)).draw().unwrap();
                            
                        let compare_with = data[0].clone();
                        for idx in 1..data.len(){
                            let current_data = data[idx].clone();
                            let ratio: Vec<(f64, f64)> = compare_with.iter().zip(current_data.iter()).map(|(&d, &di)| (di.0, di.1/d.1)).collect();
                            low_chart.draw_series(LineSeries::new(ratio, Palette99::pick(idx).filled()
                            .stroke_width(3)))
                            .unwrap();
                        }
                        // save svg to file
                        

                        /*
                        low_chart
                            .configure_series_labels()
                            .background_style(&WHITE.mix(0.8))
                            .border_style(&WHITE)
                            .label_font(("arial", 24))
                            .draw().unwrap();
                    */}
                    }
                //break;
            });
        } else {
            println!("Could not open {}", cur_filename);
        }
    } else {
        println!("No file selected");
        println!("Usage: cargo run <filename>");
        println!("Or rusttab.exe <filename>")
    }

}

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok( io::BufReader::new(file).lines())
}

fn parse(parseme: String, settings: &mut PlotDimensions, state: &mut ReadingState ){
    match state  {
        ReadingState::Head1 => {
            if let Ok(parsed) = parseme[1..13].trim().parse(){
                settings.xdim_lo = parsed;
            }
            if let Ok(parsed) = parseme[13..24].trim().parse() {
                settings.xdim_hi = parsed;
            }
            if let Ok(parsed) = parseme[24..35].trim().parse() {
                settings.ydim_lo = parsed;
            }
            if let Ok(parsed) = parseme[35..46].trim().parse() {
                settings.ydim_hi = parsed;
            }
            *state = ReadingState::Head2;
            //println!("{}", settings.xdim_lo);
        }
        ReadingState::Head2 => {
            if let Ok(parsed) = parseme[1..13].trim().parse(){
                settings.no_curves = parsed;
            }
            if let Ok(parsed) = parseme[13..24].trim().parse(){
                settings.no_points = parsed;
            }
            if let Ok(parsed) = parseme[24..35].trim().parse(){
                settings.add_border = parsed;
            }
            if let Ok(parsed) = parseme[35..46].trim().parse(){
                settings.xy_grid = parsed;
            }
            /*
            settings.ratio_first = parseme[46..57].trim().parse().unwrap();*/
            *state = ReadingState::Xlabel;
        }
        ReadingState::Xlabel => {
            settings.xlabel = parseme;
            *state = ReadingState::Ylabel;
        }
        ReadingState::Ylabel => {
            settings.ylabel = parseme;
            *state = ReadingState::Title1;
        }
        ReadingState::Title1 => {
            settings.title1 = parseme;
            *state = ReadingState::Title2;
        }
        ReadingState::Title2 => {
            settings.title2 = parseme;
            *state = ReadingState::Line1;
        }
        ReadingState::Line1 => {
            settings.xlo = PlotDimensions::default().xlo;
            settings.xhi = PlotDimensions::default().xhi;
            settings.ylo = PlotDimensions::default().ylo;
            settings.yhi = PlotDimensions::default().yhi;
            *state = ReadingState::Line2;
        }
        ReadingState::Line2 => {
            *state = ReadingState::Title1;

        }
        ReadingState::Empty => {
            *state = ReadingState::Head1;
        }
    }
}

fn parse_point(parseme: String) -> (f64, f64) {
    let mut spl = parseme.split_whitespace();
    let x: f64 = spl.next().unwrap().parse().unwrap();
    let y: f64 = spl.next().unwrap().parse().unwrap();
    (x, y)
}
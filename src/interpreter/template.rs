use logos::{Lexer, Logos};
pub type TemplateLine = Token;

type Template = Vec<TemplateLine>;

type Section = String;
#[derive(Debug, PartialEq, Eq)]
pub struct MonthDay {
    pub month: u32,
    pub day: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub struct OnDate {
    pub date: MonthDay,
    pub text: String,
}

#[derive(Logos, Debug, PartialEq, Eq)]
#[logos(extras = String)]
#[logos(subpattern whitespace = r"(\ |\t|\n|\r)")]
#[logos(subpattern params = r"(?&whitespace)*\((.|\n)*?\)")]
#[logos(subpattern month_day = r"\d{1,2}\/\d{1,2}")]
#[logos(subpattern on_date_params = r"(?&whitespace)*\((?&month_day).*?\)")]
pub enum Token {
    #[regex(r"!copy_last_day", |lex| lex.extras.clone())]
    CopyLastDay(Section),
    #[regex(r"!copy_tasks", |lex| lex.extras.clone())]
    CopyTasks(Section),
    #[regex(r"!by_weekday(\ |\t|\n|\r)*\(([^;]*;){6}([^;]*;?)(\ |\t|\n|\r)*\)", by_weekday)]
    ByWeekday([String; 7]),
    #[regex(r"!on_date(?&on_date_params)", on_date)]
    OnDate(OnDate),
    #[regex(r"#+\ .+?\n", header_callback)]
    #[regex(r".|\n", |lex| lex.slice().to_string())]
    Literal(String),
}

fn header_callback(lex: &mut Lexer<Token>) -> String {
    lex.extras = lex.slice().trim().to_string();
    lex.slice().to_string()
}

fn by_weekday(lex: &Lexer<Token>) -> [String; 7] {
    let result = get_content_between_parentheses(lex.slice());
    let content_by_day: Vec<String> = result
        .split(';')
        .map(std::string::ToString::to_string)
        .collect();
    let content_by_day: [String; 7] = content_by_day.try_into().expect("Expected 7 days");
    content_by_day
}

fn on_date(lex: &Lexer<Token>) -> OnDate {
    let result = get_content_between_parentheses(lex.slice());
    let (date_expected, text) = result
        .split_once(',')
        .expect("Expected date on '(dd/mm)' format");
    let (day, month) = date_expected
        .trim()
        .split_once('/')
        .expect("Expected month/day format");
    let month: u32 = month.parse().expect("Expected month to be a number");
    let day: u32 = day.parse().expect("Expected day to be a number");
    OnDate {
        date: MonthDay { month, day },
        text: text.trim().to_string(),
    }
}

pub fn parse(template: &str) -> Template {
    let mut result = vec![];
    for token in Token::lexer(template).flatten() {
        result.push(token);
    }
    result
}

fn get_content_between_parentheses(line: &str) -> String {
    let start_bytes = line
        .find('(')
        .expect("no opening brackets found in by weekday template line");
    let end_bytes = line
        .find(')')
        .expect("no closing brackets found in by weekday template line");
    let result = &line[start_bytes + 1..end_bytes];
    result.to_string()
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn content_between_parenthesis() {
        assert_eq!(
            get_content_between_parentheses("teste(test)teste"),
            "test".to_string()
        );
    }

    #[test]
    fn can_parse_by_weekday() {
        let expected = vec![Token::ByWeekday([
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
            "4".to_string(),
            "5".to_string(),
            "6".to_string(),
            "7".to_string(),
        ])];
        assert_eq!(parse("!by_weekday(1;2;3;4;5;6;7)"), expected);
    }
    #[test]
    fn can_parse_copy_last_day() {
        let expected = vec![
            Token::Literal("# sec\n".to_string()),
            Token::CopyLastDay("# sec".to_string()),
        ];
        assert_eq!(parse("# sec\n!copy_last_day"), expected);
    }

    #[test]
    fn can_parse_copy_tasks() {
        let expected = vec![
            Token::Literal("# sec\n".to_string()),
            Token::CopyTasks("# sec".to_string()),
        ];
        assert_eq!(parse("# sec\n!copy_tasks"), expected);
    }
    #[test]
    fn can_parse_on_date() {
        let expected = Token::OnDate(OnDate {
            date: MonthDay { month: 11, day: 20 },
            text: "test".to_string(),
        });
        assert_eq!(parse("!on_date(20/11, test)"), vec![expected]);
    }
    #[test]
    fn can_parse_literal() {
        let expected = vec![
            Token::Literal("L".to_string()),
            Token::Literal("i".to_string()),
            Token::Literal("t".to_string()),
            Token::Literal("e".to_string()),
            Token::Literal("r".to_string()),
            Token::Literal("a".to_string()),
            Token::Literal("l".to_string()),
        ];
        assert_eq!(parse("Literal"), expected);
    }
    #[test]
    fn on_full_template() {
        let mut template = String::new();
        let _ = File::open("data/dnt.md")
            .unwrap().read_to_string(&mut template);
        let output = parse(&template);
        let expected = vec![
            Token::Literal("## Quotes\n".to_string()),
            Token::CopyLastDay("## Quotes".to_string()),
            Token::Literal("\n".to_string()),
            Token::Literal("## Plans\n".to_string()),
            Token::CopyLastDay("## Plans".to_string()),
            Token::Literal("\n".to_string()),
            Token::OnDate(OnDate {
                date: MonthDay { month: 09, day: 28 },
                text: "meu Aniversário de programadora".to_string(),
            }),
            Token::Literal("\n".to_string()),
            Token::OnDate(OnDate {
                date: MonthDay { month: 03, day: 12 },
                text: "checkup".to_string(),
            }),
            Token::Literal("\n".to_string()),
            Token::OnDate(OnDate {
                date: MonthDay { month: 11, day: 24 },
                text: "test".to_string(),
            }),
            Token::Literal("\n".to_string()),
            Token::Literal("## Get a job\n".to_string()),
            Token::ByWeekday([
                "\n    ".to_string(),
                "\n    - [ ] Linkedin post".to_string(),
                "\n    - [ ] Atualizar site".to_string(),
                "\n    - [ ] Github update".to_string(),
                "\n    - [ ] Atualizar currículo".to_string(),
                "\n    - [ ] Revisar meus esforços da semana".to_string(),
                "\n".to_string(),
            ]),
            Token::Literal("\n".to_string()),
            Token::CopyTasks("## Get a job".to_string()),
            Token::Literal("### Esforços da semana\n".to_string()),
            Token::Literal("## Cronograma\n".to_string()),
            Token::ByWeekday([
                "".to_string(),
                "".to_string(),
                "".to_string(),
                "aula de violão às 16:00".to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string()
            ]),
            Token::OnDate(OnDate {
                date: MonthDay { month: 11, day: 20 },
                text: "17:00 - dentista".to_string(),
            }),
            Token::OnDate(OnDate {
                date: MonthDay { month: 12, day: 01 },
                text: "amanhã 7:30 tirar os pontos da tiramisu".to_string(),
            }),
            Token::Literal("## Habit tracker\n".to_string()),
            Token::Literal("-".to_string()),
            Token::Literal(" ".to_string()),
            Token::Literal("[".to_string()),
            Token::Literal(" ".to_string()),
            Token::Literal("]".to_string()),
            Token::Literal(" ".to_string()),
            Token::Literal("O".to_string()),
            Token::Literal("p".to_string()),
            Token::Literal("e".to_string()),
            Token::Literal("n".to_string()),
            Token::Literal(" ".to_string()),
            Token::Literal("n".to_string()),
            Token::Literal("o".to_string()),
            Token::Literal("t".to_string()),
            Token::Literal("e".to_string()),
            Token::Literal("s".to_string()),
            Token::ByWeekday([
                "\n    - [ ] long run;".to_string(),
                "\n    - [ ] short run".to_string(),
                "".to_string(),
                "".to_string(),
                "\n    - [ ] long run;".to_string(),
                "\n    - [ ] Arrumar quarto;".to_string(),
                "\n    - [ ] short run".to_string(),
            ]),
            Token::Literal("## Qual a MVA do dia\n".to_string()),
            Token::Literal("## Diary\n".to_string()),
        ];
        for i in 0..expected.len() {
            assert_eq!(output[i], expected[i]);
        }
    }
}

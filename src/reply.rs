use bytes::BytesMut;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::bytes::complete::{take_while1, take_while_m_n};
use nom::combinator::map;
use nom::multi::many_m_n;
use nom::sequence::delimited;
use nom::IResult;

#[derive(Debug)]
pub enum Reply {
    SingleLine(String),
    Err(String),
    Int(u32),
    Batch(Option<String>),
    MultiBatch(Vec<Reply>),
    BadReply,
}

fn parse_single_line(i: &str) -> IResult<&str, Reply> {
    let (i, resp) = delimited(
        tag("+"),
        take_while1(|c| c != '\r' && c != '\n'),
        tag("\r\n"),
    )(i)?;
    Ok((i, Reply::SingleLine(String::from(resp))))
}

fn parse_err(i: &str) -> IResult<&str, Reply> {
    let (i, resp) = delimited(
        tag("-"),
        take_while1(|c| c != '\r' && c != '\n'),
        tag("\r\n"),
    )(i)?;
    Ok((i, Reply::Err(String::from(resp))))
}

fn parse_int(i: &str) -> IResult<&str, Reply> {
    let (i, int) = delimited(
        tag(":"),
        map(take_while1(|c: char| c.is_digit(10)), |int: &str| {
            int.parse::<u32>().unwrap()
        }),
        tag("\r\n"),
    )(i)?;
    Ok((i, Reply::Int(int)))
}

fn parse_batch(i: &str) -> IResult<&str, Reply> {
    let (i, _) = tag("$")(i)?;
    //TODO 调整为正确的512MB大小
    let (i, len) = (take_while1(|c: char| c.is_digit(10) || c == '-'))(i)?;
    if len == "-1" {
        let (i, _) = tag("\r\n")(i)?;
        Ok((i, Reply::Batch(None)))
    } else {
        let len = len.parse::<usize>().unwrap();
        let (i, _) = tag("\r\n")(i)?;
        let (i, resp) = take_while_m_n(len, len, |_| true)(i)?;
        let (i, _) = tag("\r\n")(i)?;
        Ok((i, Reply::Batch(Some(String::from(resp)))))
    }
}

fn parse_multi_batch(i: &str) -> IResult<&str, Reply> {
    let (i, count) = delimited(
        tag("*"),
        map(take_while1(|c: char| c.is_digit(10)), |argc: &str| {
            argc.parse::<usize>().unwrap()
        }),
        tag("\r\n"),
    )(i)?;
    let (i, responses) = many_m_n(
        count,
        count,
        alt((parse_single_line, parse_err, parse_int, parse_batch)),
    )(i)?;
    if responses.len() != count {
        Ok((i, Reply::BadReply))
    } else {
        Ok((i, Reply::MultiBatch(responses)))
    }
}

fn parse(i: &str) -> IResult<&str, Reply> {
    alt((
        parse_single_line,
        parse_err,
        parse_int,
        parse_batch,
        parse_multi_batch,
    ))(i)
}

impl Reply {
    pub fn from_resp(src: &BytesMut) -> Self {
        log::debug!("{:?}", src);
        match parse(&String::from_utf8(src.as_ref().to_vec()).unwrap()) {
            Ok((_, resp)) => resp,
            Err(e) => {
                log::error!("{:?}", e);
                Reply::BadReply
            }
        }
    }
}

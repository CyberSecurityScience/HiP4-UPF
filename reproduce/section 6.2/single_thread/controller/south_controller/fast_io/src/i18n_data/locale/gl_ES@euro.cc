﻿#include"../localedef.h"

namespace fast_io_i18n
{
namespace
{

inline constexpr std::size_t monetary_mon_grouping_storage[]{3};

inline constexpr lc_all lc_all_global{.identification={.name=tsc("gl_ES@euro"),.encoding=tsc(FAST_IO_LOCALE_ENCODING),.title=tsc("Galician locale for Spain with Euro"),.source=tsc("Free Software Foundation, Inc.\t\t;\t\tfast_io"),.address=tsc("https://www.gnu.org/software/libc/\t\t;\t\thttps://gitee.com/qabeowjbtkwb/fast_io\t\t;\t\thttps://github.com/cppfastio/fast_io"),.contact=tsc("fast_io"),.email=tsc("bug-glibc-locales@gnu.org;euloanty@live.com"),.tel=tsc(""),.fax=tsc(""),.language=tsc("Galician"),.territory=tsc("Spain"),.revision=tsc("1.0"),.date=tsc("2000-08-21")},.monetary={.int_curr_symbol=tsc("EUR "),.currency_symbol=tsc("€"),.mon_decimal_point=tsc(","),.mon_thousands_sep=tsc("."),.mon_grouping={monetary_mon_grouping_storage,1},.positive_sign=tsc(""),.negative_sign=tsc("-"),.int_frac_digits=2,.frac_digits=2,.p_cs_precedes=1,.p_sep_by_space=1,.n_cs_precedes=1,.n_sep_by_space=1,.p_sign_posn=1,.n_sign_posn=1},.numeric={.decimal_point=tsc(","),.thousands_sep=tsc("")},.time={.abday={tsc("Dom"),tsc("Lun"),tsc("Mar"),tsc("Mér"),tsc("Xov"),tsc("Ven"),tsc("Sáb")},.day={tsc("Domingo"),tsc("Luns"),tsc("Martes"),tsc("Mércores"),tsc("Xoves"),tsc("Venres"),tsc("Sábado")},.abmon={tsc("Xan"),tsc("Feb"),tsc("Mar"),tsc("Abr"),tsc("Mai"),tsc("Xuñ"),tsc("Xul"),tsc("Ago"),tsc("Set"),tsc("Out"),tsc("Nov"),tsc("Dec")},.mon={tsc("Xaneiro"),tsc("Febreiro"),tsc("Marzo"),tsc("Abril"),tsc("Maio"),tsc("Xuño"),tsc("Xullo"),tsc("Agosto"),tsc("Setembro"),tsc("Outubro"),tsc("Novembro"),tsc("Decembro")},.d_t_fmt=tsc("%a %d %b %Y %T"),.d_fmt=tsc("%d//%m//%y"),.t_fmt=tsc("%T"),.t_fmt_ampm=tsc(""),.date_fmt=tsc("%a %d %b %Y %T %Z"),.am_pm={tsc(""),tsc("")},.week={7,19971130,4},.first_weekday=2},.messages={.yesexpr=tsc("^[+1sSyY]"),.noexpr=tsc("^[-0nN]"),.yesstr=tsc("si"),.nostr=tsc("non")},.paper={.width=210,.height=297},.telephone={.tel_int_fmt=tsc("+%c %a %l"),.int_select=tsc("00"),.int_prefix=tsc("34")},.name={.name_fmt=tsc("%d%t%g%t%m%t%f")},.address={.postal_fmt=tsc("%f%N%a%N%d%N%b%N%s %h %e %r%N%z %T%N%c%N"),.country_name=tsc("España"),.country_ab2=tsc("ES"),.country_ab3=tsc("ESP"),.country_num=724,.country_car=tsc("E"),.lang_name=tsc("galego"),.lang_ab=tsc("gl"),.lang_term=tsc("glg"),.lang_lib=tsc("glg")},.measurement={.measurement=1}};

inline constexpr wlc_all wlc_all_global{.identification={.name=tsc(L"gl_ES@euro"),.encoding=tsc(FAST_IO_LOCALE_LENCODING),.title=tsc(L"Galician locale for Spain with Euro"),.source=tsc(L"Free Software Foundation, Inc.\t\t;\t\tfast_io"),.address=tsc(L"https://www.gnu.org/software/libc/\t\t;\t\thttps://gitee.com/qabeowjbtkwb/fast_io\t\t;\t\thttps://github.com/cppfastio/fast_io"),.contact=tsc(L"fast_io"),.email=tsc(L"bug-glibc-locales@gnu.org;euloanty@live.com"),.tel=tsc(L""),.fax=tsc(L""),.language=tsc(L"Galician"),.territory=tsc(L"Spain"),.revision=tsc(L"1.0"),.date=tsc(L"2000-08-21")},.monetary={.int_curr_symbol=tsc(L"EUR "),.currency_symbol=tsc(L"€"),.mon_decimal_point=tsc(L","),.mon_thousands_sep=tsc(L"."),.mon_grouping={monetary_mon_grouping_storage,1},.positive_sign=tsc(L""),.negative_sign=tsc(L"-"),.int_frac_digits=2,.frac_digits=2,.p_cs_precedes=1,.p_sep_by_space=1,.n_cs_precedes=1,.n_sep_by_space=1,.p_sign_posn=1,.n_sign_posn=1},.numeric={.decimal_point=tsc(L","),.thousands_sep=tsc(L"")},.time={.abday={tsc(L"Dom"),tsc(L"Lun"),tsc(L"Mar"),tsc(L"Mér"),tsc(L"Xov"),tsc(L"Ven"),tsc(L"Sáb")},.day={tsc(L"Domingo"),tsc(L"Luns"),tsc(L"Martes"),tsc(L"Mércores"),tsc(L"Xoves"),tsc(L"Venres"),tsc(L"Sábado")},.abmon={tsc(L"Xan"),tsc(L"Feb"),tsc(L"Mar"),tsc(L"Abr"),tsc(L"Mai"),tsc(L"Xuñ"),tsc(L"Xul"),tsc(L"Ago"),tsc(L"Set"),tsc(L"Out"),tsc(L"Nov"),tsc(L"Dec")},.mon={tsc(L"Xaneiro"),tsc(L"Febreiro"),tsc(L"Marzo"),tsc(L"Abril"),tsc(L"Maio"),tsc(L"Xuño"),tsc(L"Xullo"),tsc(L"Agosto"),tsc(L"Setembro"),tsc(L"Outubro"),tsc(L"Novembro"),tsc(L"Decembro")},.d_t_fmt=tsc(L"%a %d %b %Y %T"),.d_fmt=tsc(L"%d//%m//%y"),.t_fmt=tsc(L"%T"),.t_fmt_ampm=tsc(L""),.date_fmt=tsc(L"%a %d %b %Y %T %Z"),.am_pm={tsc(L""),tsc(L"")},.week={7,19971130,4},.first_weekday=2},.messages={.yesexpr=tsc(L"^[+1sSyY]"),.noexpr=tsc(L"^[-0nN]"),.yesstr=tsc(L"si"),.nostr=tsc(L"non")},.paper={.width=210,.height=297},.telephone={.tel_int_fmt=tsc(L"+%c %a %l"),.int_select=tsc(L"00"),.int_prefix=tsc(L"34")},.name={.name_fmt=tsc(L"%d%t%g%t%m%t%f")},.address={.postal_fmt=tsc(L"%f%N%a%N%d%N%b%N%s %h %e %r%N%z %T%N%c%N"),.country_name=tsc(L"España"),.country_ab2=tsc(L"ES"),.country_ab3=tsc(L"ESP"),.country_num=724,.country_car=tsc(L"E"),.lang_name=tsc(L"galego"),.lang_ab=tsc(L"gl"),.lang_term=tsc(L"glg"),.lang_lib=tsc(L"glg")},.measurement={.measurement=1}};

inline constexpr u8lc_all u8lc_all_global{.identification={.name=tsc(u8"gl_ES@euro"),.encoding=tsc(FAST_IO_LOCALE_u8ENCODING),.title=tsc(u8"Galician locale for Spain with Euro"),.source=tsc(u8"Free Software Foundation, Inc.\t\t;\t\tfast_io"),.address=tsc(u8"https://www.gnu.org/software/libc/\t\t;\t\thttps://gitee.com/qabeowjbtkwb/fast_io\t\t;\t\thttps://github.com/cppfastio/fast_io"),.contact=tsc(u8"fast_io"),.email=tsc(u8"bug-glibc-locales@gnu.org;euloanty@live.com"),.tel=tsc(u8""),.fax=tsc(u8""),.language=tsc(u8"Galician"),.territory=tsc(u8"Spain"),.revision=tsc(u8"1.0"),.date=tsc(u8"2000-08-21")},.monetary={.int_curr_symbol=tsc(u8"EUR "),.currency_symbol=tsc(u8"€"),.mon_decimal_point=tsc(u8","),.mon_thousands_sep=tsc(u8"."),.mon_grouping={monetary_mon_grouping_storage,1},.positive_sign=tsc(u8""),.negative_sign=tsc(u8"-"),.int_frac_digits=2,.frac_digits=2,.p_cs_precedes=1,.p_sep_by_space=1,.n_cs_precedes=1,.n_sep_by_space=1,.p_sign_posn=1,.n_sign_posn=1},.numeric={.decimal_point=tsc(u8","),.thousands_sep=tsc(u8"")},.time={.abday={tsc(u8"Dom"),tsc(u8"Lun"),tsc(u8"Mar"),tsc(u8"Mér"),tsc(u8"Xov"),tsc(u8"Ven"),tsc(u8"Sáb")},.day={tsc(u8"Domingo"),tsc(u8"Luns"),tsc(u8"Martes"),tsc(u8"Mércores"),tsc(u8"Xoves"),tsc(u8"Venres"),tsc(u8"Sábado")},.abmon={tsc(u8"Xan"),tsc(u8"Feb"),tsc(u8"Mar"),tsc(u8"Abr"),tsc(u8"Mai"),tsc(u8"Xuñ"),tsc(u8"Xul"),tsc(u8"Ago"),tsc(u8"Set"),tsc(u8"Out"),tsc(u8"Nov"),tsc(u8"Dec")},.mon={tsc(u8"Xaneiro"),tsc(u8"Febreiro"),tsc(u8"Marzo"),tsc(u8"Abril"),tsc(u8"Maio"),tsc(u8"Xuño"),tsc(u8"Xullo"),tsc(u8"Agosto"),tsc(u8"Setembro"),tsc(u8"Outubro"),tsc(u8"Novembro"),tsc(u8"Decembro")},.d_t_fmt=tsc(u8"%a %d %b %Y %T"),.d_fmt=tsc(u8"%d//%m//%y"),.t_fmt=tsc(u8"%T"),.t_fmt_ampm=tsc(u8""),.date_fmt=tsc(u8"%a %d %b %Y %T %Z"),.am_pm={tsc(u8""),tsc(u8"")},.week={7,19971130,4},.first_weekday=2},.messages={.yesexpr=tsc(u8"^[+1sSyY]"),.noexpr=tsc(u8"^[-0nN]"),.yesstr=tsc(u8"si"),.nostr=tsc(u8"non")},.paper={.width=210,.height=297},.telephone={.tel_int_fmt=tsc(u8"+%c %a %l"),.int_select=tsc(u8"00"),.int_prefix=tsc(u8"34")},.name={.name_fmt=tsc(u8"%d%t%g%t%m%t%f")},.address={.postal_fmt=tsc(u8"%f%N%a%N%d%N%b%N%s %h %e %r%N%z %T%N%c%N"),.country_name=tsc(u8"España"),.country_ab2=tsc(u8"ES"),.country_ab3=tsc(u8"ESP"),.country_num=724,.country_car=tsc(u8"E"),.lang_name=tsc(u8"galego"),.lang_ab=tsc(u8"gl"),.lang_term=tsc(u8"glg"),.lang_lib=tsc(u8"glg")},.measurement={.measurement=1}};

inline constexpr u16lc_all u16lc_all_global{.identification={.name=tsc(u"gl_ES@euro"),.encoding=tsc(FAST_IO_LOCALE_uENCODING),.title=tsc(u"Galician locale for Spain with Euro"),.source=tsc(u"Free Software Foundation, Inc.\t\t;\t\tfast_io"),.address=tsc(u"https://www.gnu.org/software/libc/\t\t;\t\thttps://gitee.com/qabeowjbtkwb/fast_io\t\t;\t\thttps://github.com/cppfastio/fast_io"),.contact=tsc(u"fast_io"),.email=tsc(u"bug-glibc-locales@gnu.org;euloanty@live.com"),.tel=tsc(u""),.fax=tsc(u""),.language=tsc(u"Galician"),.territory=tsc(u"Spain"),.revision=tsc(u"1.0"),.date=tsc(u"2000-08-21")},.monetary={.int_curr_symbol=tsc(u"EUR "),.currency_symbol=tsc(u"€"),.mon_decimal_point=tsc(u","),.mon_thousands_sep=tsc(u"."),.mon_grouping={monetary_mon_grouping_storage,1},.positive_sign=tsc(u""),.negative_sign=tsc(u"-"),.int_frac_digits=2,.frac_digits=2,.p_cs_precedes=1,.p_sep_by_space=1,.n_cs_precedes=1,.n_sep_by_space=1,.p_sign_posn=1,.n_sign_posn=1},.numeric={.decimal_point=tsc(u","),.thousands_sep=tsc(u"")},.time={.abday={tsc(u"Dom"),tsc(u"Lun"),tsc(u"Mar"),tsc(u"Mér"),tsc(u"Xov"),tsc(u"Ven"),tsc(u"Sáb")},.day={tsc(u"Domingo"),tsc(u"Luns"),tsc(u"Martes"),tsc(u"Mércores"),tsc(u"Xoves"),tsc(u"Venres"),tsc(u"Sábado")},.abmon={tsc(u"Xan"),tsc(u"Feb"),tsc(u"Mar"),tsc(u"Abr"),tsc(u"Mai"),tsc(u"Xuñ"),tsc(u"Xul"),tsc(u"Ago"),tsc(u"Set"),tsc(u"Out"),tsc(u"Nov"),tsc(u"Dec")},.mon={tsc(u"Xaneiro"),tsc(u"Febreiro"),tsc(u"Marzo"),tsc(u"Abril"),tsc(u"Maio"),tsc(u"Xuño"),tsc(u"Xullo"),tsc(u"Agosto"),tsc(u"Setembro"),tsc(u"Outubro"),tsc(u"Novembro"),tsc(u"Decembro")},.d_t_fmt=tsc(u"%a %d %b %Y %T"),.d_fmt=tsc(u"%d//%m//%y"),.t_fmt=tsc(u"%T"),.t_fmt_ampm=tsc(u""),.date_fmt=tsc(u"%a %d %b %Y %T %Z"),.am_pm={tsc(u""),tsc(u"")},.week={7,19971130,4},.first_weekday=2},.messages={.yesexpr=tsc(u"^[+1sSyY]"),.noexpr=tsc(u"^[-0nN]"),.yesstr=tsc(u"si"),.nostr=tsc(u"non")},.paper={.width=210,.height=297},.telephone={.tel_int_fmt=tsc(u"+%c %a %l"),.int_select=tsc(u"00"),.int_prefix=tsc(u"34")},.name={.name_fmt=tsc(u"%d%t%g%t%m%t%f")},.address={.postal_fmt=tsc(u"%f%N%a%N%d%N%b%N%s %h %e %r%N%z %T%N%c%N"),.country_name=tsc(u"España"),.country_ab2=tsc(u"ES"),.country_ab3=tsc(u"ESP"),.country_num=724,.country_car=tsc(u"E"),.lang_name=tsc(u"galego"),.lang_ab=tsc(u"gl"),.lang_term=tsc(u"glg"),.lang_lib=tsc(u"glg")},.measurement={.measurement=1}};

inline constexpr u32lc_all u32lc_all_global{.identification={.name=tsc(U"gl_ES@euro"),.encoding=tsc(FAST_IO_LOCALE_UENCODING),.title=tsc(U"Galician locale for Spain with Euro"),.source=tsc(U"Free Software Foundation, Inc.\t\t;\t\tfast_io"),.address=tsc(U"https://www.gnu.org/software/libc/\t\t;\t\thttps://gitee.com/qabeowjbtkwb/fast_io\t\t;\t\thttps://github.com/cppfastio/fast_io"),.contact=tsc(U"fast_io"),.email=tsc(U"bug-glibc-locales@gnu.org;euloanty@live.com"),.tel=tsc(U""),.fax=tsc(U""),.language=tsc(U"Galician"),.territory=tsc(U"Spain"),.revision=tsc(U"1.0"),.date=tsc(U"2000-08-21")},.monetary={.int_curr_symbol=tsc(U"EUR "),.currency_symbol=tsc(U"€"),.mon_decimal_point=tsc(U","),.mon_thousands_sep=tsc(U"."),.mon_grouping={monetary_mon_grouping_storage,1},.positive_sign=tsc(U""),.negative_sign=tsc(U"-"),.int_frac_digits=2,.frac_digits=2,.p_cs_precedes=1,.p_sep_by_space=1,.n_cs_precedes=1,.n_sep_by_space=1,.p_sign_posn=1,.n_sign_posn=1},.numeric={.decimal_point=tsc(U","),.thousands_sep=tsc(U"")},.time={.abday={tsc(U"Dom"),tsc(U"Lun"),tsc(U"Mar"),tsc(U"Mér"),tsc(U"Xov"),tsc(U"Ven"),tsc(U"Sáb")},.day={tsc(U"Domingo"),tsc(U"Luns"),tsc(U"Martes"),tsc(U"Mércores"),tsc(U"Xoves"),tsc(U"Venres"),tsc(U"Sábado")},.abmon={tsc(U"Xan"),tsc(U"Feb"),tsc(U"Mar"),tsc(U"Abr"),tsc(U"Mai"),tsc(U"Xuñ"),tsc(U"Xul"),tsc(U"Ago"),tsc(U"Set"),tsc(U"Out"),tsc(U"Nov"),tsc(U"Dec")},.mon={tsc(U"Xaneiro"),tsc(U"Febreiro"),tsc(U"Marzo"),tsc(U"Abril"),tsc(U"Maio"),tsc(U"Xuño"),tsc(U"Xullo"),tsc(U"Agosto"),tsc(U"Setembro"),tsc(U"Outubro"),tsc(U"Novembro"),tsc(U"Decembro")},.d_t_fmt=tsc(U"%a %d %b %Y %T"),.d_fmt=tsc(U"%d//%m//%y"),.t_fmt=tsc(U"%T"),.t_fmt_ampm=tsc(U""),.date_fmt=tsc(U"%a %d %b %Y %T %Z"),.am_pm={tsc(U""),tsc(U"")},.week={7,19971130,4},.first_weekday=2},.messages={.yesexpr=tsc(U"^[+1sSyY]"),.noexpr=tsc(U"^[-0nN]"),.yesstr=tsc(U"si"),.nostr=tsc(U"non")},.paper={.width=210,.height=297},.telephone={.tel_int_fmt=tsc(U"+%c %a %l"),.int_select=tsc(U"00"),.int_prefix=tsc(U"34")},.name={.name_fmt=tsc(U"%d%t%g%t%m%t%f")},.address={.postal_fmt=tsc(U"%f%N%a%N%d%N%b%N%s %h %e %r%N%z %T%N%c%N"),.country_name=tsc(U"España"),.country_ab2=tsc(U"ES"),.country_ab3=tsc(U"ESP"),.country_num=724,.country_car=tsc(U"E"),.lang_name=tsc(U"galego"),.lang_ab=tsc(U"gl"),.lang_term=tsc(U"glg"),.lang_lib=tsc(U"glg")},.measurement={.measurement=1}};


}
}

#include"../main.h"
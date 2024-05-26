﻿#include"../localedef.h"

namespace fast_io_i18n
{
namespace
{

inline constexpr std::size_t numeric_grouping_storage[]{3};

inline constexpr lc_all lc_all_global{.identification={.name=tsc("ru_UA"),.encoding=tsc(FAST_IO_LOCALE_ENCODING),.title=tsc("Russian locale for Ukraine"),.source=tsc("RFC 2319\t\t;\t\tfast_io"),.address=tsc("https://gitee.com/qabeowjbtkwb/fast_io\t\t;\t\thttps://github.com/cppfastio/fast_io"),.contact=tsc("fast_io"),.email=tsc("bug-glibc-locales@gnu.org;euloanty@live.com"),.tel=tsc(""),.fax=tsc(""),.language=tsc("Russian"),.territory=tsc("Ukraine"),.revision=tsc("1.0"),.date=tsc("2000-06-29")},.monetary={.int_curr_symbol=tsc("UAH "),.currency_symbol=tsc("₴"),.mon_decimal_point=tsc("."),.mon_thousands_sep=tsc(" "),.mon_grouping={numeric_grouping_storage,1},.positive_sign=tsc(""),.negative_sign=tsc("-"),.int_frac_digits=2,.frac_digits=2,.p_cs_precedes=0,.p_sep_by_space=1,.n_cs_precedes=0,.n_sep_by_space=1,.p_sign_posn=1,.n_sign_posn=1},.numeric={.decimal_point=tsc(","),.thousands_sep=tsc("."),.grouping={numeric_grouping_storage,1}},.time={.abday={tsc("Вс"),tsc("Пн"),tsc("Вт"),tsc("Ср"),tsc("Чт"),tsc("Пт"),tsc("Сб")},.day={tsc("Воскресенье"),tsc("Понедельник"),tsc("Вторник"),tsc("Среда"),tsc("Четверг"),tsc("Пятница"),tsc("Суббота")},.abmon={tsc("янв"),tsc("фев"),tsc("мар"),tsc("апр"),tsc("мая"),tsc("июн"),tsc("июл"),tsc("авг"),tsc("сен"),tsc("окт"),tsc("ноя"),tsc("дек")},.ab_alt_mon={tsc("янв"),tsc("фев"),tsc("мар"),tsc("апр"),tsc("май"),tsc("июн"),tsc("июл"),tsc("авг"),tsc("сен"),tsc("окт"),tsc("ноя"),tsc("дек")},.mon={tsc("января"),tsc("февраля"),tsc("марта"),tsc("апреля"),tsc("мая"),tsc("июня"),tsc("июля"),tsc("августа"),tsc("сентября"),tsc("октября"),tsc("ноября"),tsc("декабря")},.d_t_fmt=tsc("%a %d %b %Y %T"),.d_fmt=tsc("%d.%m.%Y"),.t_fmt=tsc("%T"),.t_fmt_ampm=tsc(""),.date_fmt=tsc("%a %d %b %Y %T %Z"),.am_pm={tsc(""),tsc("")},.week={7,19971130,1},.first_weekday=2},.messages={.yesexpr=tsc("^[+1yYДд]"),.noexpr=tsc("^[-0nNНн]"),.yesstr=tsc("да"),.nostr=tsc("нет")},.paper={.width=210,.height=297},.telephone={.tel_int_fmt=tsc("+%c %a %l"),.int_select=tsc("00"),.int_prefix=tsc("380")},.name={.name_fmt=tsc("%d%t%g%t%m%t%f")},.address={.postal_fmt=tsc("%f%N%a%N%d%N%b%N%s %h %e %r%N%z %T%N%c%N"),.country_name=tsc("Украина"),.country_ab2=tsc("UA"),.country_ab3=tsc("UKR"),.country_num=804,.country_car=tsc("UA"),.lang_name=tsc("русский"),.lang_ab=tsc("ru"),.lang_term=tsc("rus"),.lang_lib=tsc("rus")},.measurement={.measurement=1}};

inline constexpr wlc_all wlc_all_global{.identification={.name=tsc(L"ru_UA"),.encoding=tsc(FAST_IO_LOCALE_LENCODING),.title=tsc(L"Russian locale for Ukraine"),.source=tsc(L"RFC 2319\t\t;\t\tfast_io"),.address=tsc(L"https://gitee.com/qabeowjbtkwb/fast_io\t\t;\t\thttps://github.com/cppfastio/fast_io"),.contact=tsc(L"fast_io"),.email=tsc(L"bug-glibc-locales@gnu.org;euloanty@live.com"),.tel=tsc(L""),.fax=tsc(L""),.language=tsc(L"Russian"),.territory=tsc(L"Ukraine"),.revision=tsc(L"1.0"),.date=tsc(L"2000-06-29")},.monetary={.int_curr_symbol=tsc(L"UAH "),.currency_symbol=tsc(L"₴"),.mon_decimal_point=tsc(L"."),.mon_thousands_sep=tsc(L" "),.mon_grouping={numeric_grouping_storage,1},.positive_sign=tsc(L""),.negative_sign=tsc(L"-"),.int_frac_digits=2,.frac_digits=2,.p_cs_precedes=0,.p_sep_by_space=1,.n_cs_precedes=0,.n_sep_by_space=1,.p_sign_posn=1,.n_sign_posn=1},.numeric={.decimal_point=tsc(L","),.thousands_sep=tsc(L"."),.grouping={numeric_grouping_storage,1}},.time={.abday={tsc(L"Вс"),tsc(L"Пн"),tsc(L"Вт"),tsc(L"Ср"),tsc(L"Чт"),tsc(L"Пт"),tsc(L"Сб")},.day={tsc(L"Воскресенье"),tsc(L"Понедельник"),tsc(L"Вторник"),tsc(L"Среда"),tsc(L"Четверг"),tsc(L"Пятница"),tsc(L"Суббота")},.abmon={tsc(L"янв"),tsc(L"фев"),tsc(L"мар"),tsc(L"апр"),tsc(L"мая"),tsc(L"июн"),tsc(L"июл"),tsc(L"авг"),tsc(L"сен"),tsc(L"окт"),tsc(L"ноя"),tsc(L"дек")},.ab_alt_mon={tsc(L"янв"),tsc(L"фев"),tsc(L"мар"),tsc(L"апр"),tsc(L"май"),tsc(L"июн"),tsc(L"июл"),tsc(L"авг"),tsc(L"сен"),tsc(L"окт"),tsc(L"ноя"),tsc(L"дек")},.mon={tsc(L"января"),tsc(L"февраля"),tsc(L"марта"),tsc(L"апреля"),tsc(L"мая"),tsc(L"июня"),tsc(L"июля"),tsc(L"августа"),tsc(L"сентября"),tsc(L"октября"),tsc(L"ноября"),tsc(L"декабря")},.d_t_fmt=tsc(L"%a %d %b %Y %T"),.d_fmt=tsc(L"%d.%m.%Y"),.t_fmt=tsc(L"%T"),.t_fmt_ampm=tsc(L""),.date_fmt=tsc(L"%a %d %b %Y %T %Z"),.am_pm={tsc(L""),tsc(L"")},.week={7,19971130,1},.first_weekday=2},.messages={.yesexpr=tsc(L"^[+1yYДд]"),.noexpr=tsc(L"^[-0nNНн]"),.yesstr=tsc(L"да"),.nostr=tsc(L"нет")},.paper={.width=210,.height=297},.telephone={.tel_int_fmt=tsc(L"+%c %a %l"),.int_select=tsc(L"00"),.int_prefix=tsc(L"380")},.name={.name_fmt=tsc(L"%d%t%g%t%m%t%f")},.address={.postal_fmt=tsc(L"%f%N%a%N%d%N%b%N%s %h %e %r%N%z %T%N%c%N"),.country_name=tsc(L"Украина"),.country_ab2=tsc(L"UA"),.country_ab3=tsc(L"UKR"),.country_num=804,.country_car=tsc(L"UA"),.lang_name=tsc(L"русский"),.lang_ab=tsc(L"ru"),.lang_term=tsc(L"rus"),.lang_lib=tsc(L"rus")},.measurement={.measurement=1}};

inline constexpr u8lc_all u8lc_all_global{.identification={.name=tsc(u8"ru_UA"),.encoding=tsc(FAST_IO_LOCALE_u8ENCODING),.title=tsc(u8"Russian locale for Ukraine"),.source=tsc(u8"RFC 2319\t\t;\t\tfast_io"),.address=tsc(u8"https://gitee.com/qabeowjbtkwb/fast_io\t\t;\t\thttps://github.com/cppfastio/fast_io"),.contact=tsc(u8"fast_io"),.email=tsc(u8"bug-glibc-locales@gnu.org;euloanty@live.com"),.tel=tsc(u8""),.fax=tsc(u8""),.language=tsc(u8"Russian"),.territory=tsc(u8"Ukraine"),.revision=tsc(u8"1.0"),.date=tsc(u8"2000-06-29")},.monetary={.int_curr_symbol=tsc(u8"UAH "),.currency_symbol=tsc(u8"₴"),.mon_decimal_point=tsc(u8"."),.mon_thousands_sep=tsc(u8" "),.mon_grouping={numeric_grouping_storage,1},.positive_sign=tsc(u8""),.negative_sign=tsc(u8"-"),.int_frac_digits=2,.frac_digits=2,.p_cs_precedes=0,.p_sep_by_space=1,.n_cs_precedes=0,.n_sep_by_space=1,.p_sign_posn=1,.n_sign_posn=1},.numeric={.decimal_point=tsc(u8","),.thousands_sep=tsc(u8"."),.grouping={numeric_grouping_storage,1}},.time={.abday={tsc(u8"Вс"),tsc(u8"Пн"),tsc(u8"Вт"),tsc(u8"Ср"),tsc(u8"Чт"),tsc(u8"Пт"),tsc(u8"Сб")},.day={tsc(u8"Воскресенье"),tsc(u8"Понедельник"),tsc(u8"Вторник"),tsc(u8"Среда"),tsc(u8"Четверг"),tsc(u8"Пятница"),tsc(u8"Суббота")},.abmon={tsc(u8"янв"),tsc(u8"фев"),tsc(u8"мар"),tsc(u8"апр"),tsc(u8"мая"),tsc(u8"июн"),tsc(u8"июл"),tsc(u8"авг"),tsc(u8"сен"),tsc(u8"окт"),tsc(u8"ноя"),tsc(u8"дек")},.ab_alt_mon={tsc(u8"янв"),tsc(u8"фев"),tsc(u8"мар"),tsc(u8"апр"),tsc(u8"май"),tsc(u8"июн"),tsc(u8"июл"),tsc(u8"авг"),tsc(u8"сен"),tsc(u8"окт"),tsc(u8"ноя"),tsc(u8"дек")},.mon={tsc(u8"января"),tsc(u8"февраля"),tsc(u8"марта"),tsc(u8"апреля"),tsc(u8"мая"),tsc(u8"июня"),tsc(u8"июля"),tsc(u8"августа"),tsc(u8"сентября"),tsc(u8"октября"),tsc(u8"ноября"),tsc(u8"декабря")},.d_t_fmt=tsc(u8"%a %d %b %Y %T"),.d_fmt=tsc(u8"%d.%m.%Y"),.t_fmt=tsc(u8"%T"),.t_fmt_ampm=tsc(u8""),.date_fmt=tsc(u8"%a %d %b %Y %T %Z"),.am_pm={tsc(u8""),tsc(u8"")},.week={7,19971130,1},.first_weekday=2},.messages={.yesexpr=tsc(u8"^[+1yYДд]"),.noexpr=tsc(u8"^[-0nNНн]"),.yesstr=tsc(u8"да"),.nostr=tsc(u8"нет")},.paper={.width=210,.height=297},.telephone={.tel_int_fmt=tsc(u8"+%c %a %l"),.int_select=tsc(u8"00"),.int_prefix=tsc(u8"380")},.name={.name_fmt=tsc(u8"%d%t%g%t%m%t%f")},.address={.postal_fmt=tsc(u8"%f%N%a%N%d%N%b%N%s %h %e %r%N%z %T%N%c%N"),.country_name=tsc(u8"Украина"),.country_ab2=tsc(u8"UA"),.country_ab3=tsc(u8"UKR"),.country_num=804,.country_car=tsc(u8"UA"),.lang_name=tsc(u8"русский"),.lang_ab=tsc(u8"ru"),.lang_term=tsc(u8"rus"),.lang_lib=tsc(u8"rus")},.measurement={.measurement=1}};

inline constexpr u16lc_all u16lc_all_global{.identification={.name=tsc(u"ru_UA"),.encoding=tsc(FAST_IO_LOCALE_uENCODING),.title=tsc(u"Russian locale for Ukraine"),.source=tsc(u"RFC 2319\t\t;\t\tfast_io"),.address=tsc(u"https://gitee.com/qabeowjbtkwb/fast_io\t\t;\t\thttps://github.com/cppfastio/fast_io"),.contact=tsc(u"fast_io"),.email=tsc(u"bug-glibc-locales@gnu.org;euloanty@live.com"),.tel=tsc(u""),.fax=tsc(u""),.language=tsc(u"Russian"),.territory=tsc(u"Ukraine"),.revision=tsc(u"1.0"),.date=tsc(u"2000-06-29")},.monetary={.int_curr_symbol=tsc(u"UAH "),.currency_symbol=tsc(u"₴"),.mon_decimal_point=tsc(u"."),.mon_thousands_sep=tsc(u" "),.mon_grouping={numeric_grouping_storage,1},.positive_sign=tsc(u""),.negative_sign=tsc(u"-"),.int_frac_digits=2,.frac_digits=2,.p_cs_precedes=0,.p_sep_by_space=1,.n_cs_precedes=0,.n_sep_by_space=1,.p_sign_posn=1,.n_sign_posn=1},.numeric={.decimal_point=tsc(u","),.thousands_sep=tsc(u"."),.grouping={numeric_grouping_storage,1}},.time={.abday={tsc(u"Вс"),tsc(u"Пн"),tsc(u"Вт"),tsc(u"Ср"),tsc(u"Чт"),tsc(u"Пт"),tsc(u"Сб")},.day={tsc(u"Воскресенье"),tsc(u"Понедельник"),tsc(u"Вторник"),tsc(u"Среда"),tsc(u"Четверг"),tsc(u"Пятница"),tsc(u"Суббота")},.abmon={tsc(u"янв"),tsc(u"фев"),tsc(u"мар"),tsc(u"апр"),tsc(u"мая"),tsc(u"июн"),tsc(u"июл"),tsc(u"авг"),tsc(u"сен"),tsc(u"окт"),tsc(u"ноя"),tsc(u"дек")},.ab_alt_mon={tsc(u"янв"),tsc(u"фев"),tsc(u"мар"),tsc(u"апр"),tsc(u"май"),tsc(u"июн"),tsc(u"июл"),tsc(u"авг"),tsc(u"сен"),tsc(u"окт"),tsc(u"ноя"),tsc(u"дек")},.mon={tsc(u"января"),tsc(u"февраля"),tsc(u"марта"),tsc(u"апреля"),tsc(u"мая"),tsc(u"июня"),tsc(u"июля"),tsc(u"августа"),tsc(u"сентября"),tsc(u"октября"),tsc(u"ноября"),tsc(u"декабря")},.d_t_fmt=tsc(u"%a %d %b %Y %T"),.d_fmt=tsc(u"%d.%m.%Y"),.t_fmt=tsc(u"%T"),.t_fmt_ampm=tsc(u""),.date_fmt=tsc(u"%a %d %b %Y %T %Z"),.am_pm={tsc(u""),tsc(u"")},.week={7,19971130,1},.first_weekday=2},.messages={.yesexpr=tsc(u"^[+1yYДд]"),.noexpr=tsc(u"^[-0nNНн]"),.yesstr=tsc(u"да"),.nostr=tsc(u"нет")},.paper={.width=210,.height=297},.telephone={.tel_int_fmt=tsc(u"+%c %a %l"),.int_select=tsc(u"00"),.int_prefix=tsc(u"380")},.name={.name_fmt=tsc(u"%d%t%g%t%m%t%f")},.address={.postal_fmt=tsc(u"%f%N%a%N%d%N%b%N%s %h %e %r%N%z %T%N%c%N"),.country_name=tsc(u"Украина"),.country_ab2=tsc(u"UA"),.country_ab3=tsc(u"UKR"),.country_num=804,.country_car=tsc(u"UA"),.lang_name=tsc(u"русский"),.lang_ab=tsc(u"ru"),.lang_term=tsc(u"rus"),.lang_lib=tsc(u"rus")},.measurement={.measurement=1}};

inline constexpr u32lc_all u32lc_all_global{.identification={.name=tsc(U"ru_UA"),.encoding=tsc(FAST_IO_LOCALE_UENCODING),.title=tsc(U"Russian locale for Ukraine"),.source=tsc(U"RFC 2319\t\t;\t\tfast_io"),.address=tsc(U"https://gitee.com/qabeowjbtkwb/fast_io\t\t;\t\thttps://github.com/cppfastio/fast_io"),.contact=tsc(U"fast_io"),.email=tsc(U"bug-glibc-locales@gnu.org;euloanty@live.com"),.tel=tsc(U""),.fax=tsc(U""),.language=tsc(U"Russian"),.territory=tsc(U"Ukraine"),.revision=tsc(U"1.0"),.date=tsc(U"2000-06-29")},.monetary={.int_curr_symbol=tsc(U"UAH "),.currency_symbol=tsc(U"₴"),.mon_decimal_point=tsc(U"."),.mon_thousands_sep=tsc(U" "),.mon_grouping={numeric_grouping_storage,1},.positive_sign=tsc(U""),.negative_sign=tsc(U"-"),.int_frac_digits=2,.frac_digits=2,.p_cs_precedes=0,.p_sep_by_space=1,.n_cs_precedes=0,.n_sep_by_space=1,.p_sign_posn=1,.n_sign_posn=1},.numeric={.decimal_point=tsc(U","),.thousands_sep=tsc(U"."),.grouping={numeric_grouping_storage,1}},.time={.abday={tsc(U"Вс"),tsc(U"Пн"),tsc(U"Вт"),tsc(U"Ср"),tsc(U"Чт"),tsc(U"Пт"),tsc(U"Сб")},.day={tsc(U"Воскресенье"),tsc(U"Понедельник"),tsc(U"Вторник"),tsc(U"Среда"),tsc(U"Четверг"),tsc(U"Пятница"),tsc(U"Суббота")},.abmon={tsc(U"янв"),tsc(U"фев"),tsc(U"мар"),tsc(U"апр"),tsc(U"мая"),tsc(U"июн"),tsc(U"июл"),tsc(U"авг"),tsc(U"сен"),tsc(U"окт"),tsc(U"ноя"),tsc(U"дек")},.ab_alt_mon={tsc(U"янв"),tsc(U"фев"),tsc(U"мар"),tsc(U"апр"),tsc(U"май"),tsc(U"июн"),tsc(U"июл"),tsc(U"авг"),tsc(U"сен"),tsc(U"окт"),tsc(U"ноя"),tsc(U"дек")},.mon={tsc(U"января"),tsc(U"февраля"),tsc(U"марта"),tsc(U"апреля"),tsc(U"мая"),tsc(U"июня"),tsc(U"июля"),tsc(U"августа"),tsc(U"сентября"),tsc(U"октября"),tsc(U"ноября"),tsc(U"декабря")},.d_t_fmt=tsc(U"%a %d %b %Y %T"),.d_fmt=tsc(U"%d.%m.%Y"),.t_fmt=tsc(U"%T"),.t_fmt_ampm=tsc(U""),.date_fmt=tsc(U"%a %d %b %Y %T %Z"),.am_pm={tsc(U""),tsc(U"")},.week={7,19971130,1},.first_weekday=2},.messages={.yesexpr=tsc(U"^[+1yYДд]"),.noexpr=tsc(U"^[-0nNНн]"),.yesstr=tsc(U"да"),.nostr=tsc(U"нет")},.paper={.width=210,.height=297},.telephone={.tel_int_fmt=tsc(U"+%c %a %l"),.int_select=tsc(U"00"),.int_prefix=tsc(U"380")},.name={.name_fmt=tsc(U"%d%t%g%t%m%t%f")},.address={.postal_fmt=tsc(U"%f%N%a%N%d%N%b%N%s %h %e %r%N%z %T%N%c%N"),.country_name=tsc(U"Украина"),.country_ab2=tsc(U"UA"),.country_ab3=tsc(U"UKR"),.country_num=804,.country_car=tsc(U"UA"),.lang_name=tsc(U"русский"),.lang_ab=tsc(U"ru"),.lang_term=tsc(U"rus"),.lang_lib=tsc(U"rus")},.measurement={.measurement=1}};


}
}

#include"../main.h"
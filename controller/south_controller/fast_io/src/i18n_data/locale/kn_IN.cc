﻿#include"../localedef.h"

namespace fast_io_i18n
{
namespace
{

inline constexpr std::size_t monetary_mon_grouping_storage[]{3,2};

inline constexpr std::size_t numeric_grouping_storage[]{3};

inline constexpr lc_all lc_all_global{.identification={.name=tsc("kn_IN"),.encoding=tsc(FAST_IO_LOCALE_ENCODING),.title=tsc("Kannada language locale for India"),.source=tsc("IndLinux.org\t\t;\t\tfast_io"),.address=tsc("https://gitee.com/qabeowjbtkwb/fast_io\t\t;\t\thttps://github.com/cppfastio/fast_io"),.contact=tsc("fast_io"),.email=tsc("bug-glibc-locales@gnu.org;euloanty@live.com"),.tel=tsc(""),.fax=tsc(""),.language=tsc("Kannada"),.territory=tsc("India"),.revision=tsc("0.1"),.date=tsc("2002-11-28")},.monetary={.int_curr_symbol=tsc("INR "),.currency_symbol=tsc("₹"),.mon_decimal_point=tsc("."),.mon_thousands_sep=tsc(","),.mon_grouping={monetary_mon_grouping_storage,2},.positive_sign=tsc(""),.negative_sign=tsc("-"),.int_frac_digits=2,.frac_digits=2,.p_cs_precedes=1,.p_sep_by_space=0,.n_cs_precedes=1,.n_sep_by_space=0,.p_sign_posn=1,.n_sign_posn=1},.numeric={.decimal_point=tsc("."),.thousands_sep=tsc(","),.grouping={numeric_grouping_storage,1}},.time={.abday={tsc("ರ"),tsc("ಸೋ"),tsc("ಮಂ"),tsc("ಬು"),tsc("ಗು"),tsc("ಶು"),tsc("ಶ")},.day={tsc("ರವಿವಾರ"),tsc("ಸೋಮವಾರ"),tsc("ಮಂಗಳವಾರ"),tsc("ಬುಧವಾರ"),tsc("ಗುರುವಾರ"),tsc("ಶುಕ್ರವಾರ"),tsc("ಶನಿವಾರ")},.abmon={tsc("ಜನ"),tsc("ಫೆಬ್ರ"),tsc("ಮಾರ್ಚ್"),tsc("ಏಪ್ರಿ"),tsc("ಮೇ"),tsc("ಜೂನ್"),tsc("ಜುಲೈ"),tsc("ಆ"),tsc("ಸೆಪ್ಟೆಂ"),tsc("ಅಕ್ಟೋ"),tsc("ನವೆಂ"),tsc("ಡಿಸೆಂ")},.mon={tsc("ಜನವರಿ"),tsc("ಫೆಬ್ರವರಿ"),tsc("ಮಾರ್ಚ್"),tsc("ಏಪ್ರಿಲ್"),tsc("ಮೇ"),tsc("ಜೂನ್"),tsc("ಜುಲೈ"),tsc("ಆಗಸ್ಟ್"),tsc("ಸೆಪ್ಟೆಂಬರ್"),tsc("ಅಕ್ಟೋಬರ್"),tsc("ನವೆಂಬರ್"),tsc("ಡಿಸೆಂಬರ್")},.d_t_fmt=tsc("%A %d %b %Y %I:%M:%S %p"),.d_fmt=tsc("%-d//%-m//%y"),.t_fmt=tsc("%I:%M:%S %p %Z"),.t_fmt_ampm=tsc("%I:%M:%S %p %Z"),.date_fmt=tsc("%A %d %b %Y %I:%M:%S %p %Z"),.am_pm={tsc("ಪೂರ್ವಾಹ್ನ"),tsc("ಅಪರಾಹ್ನ")},.week={7,19971130,1}},.messages={.yesexpr=tsc("^[+1yYಹ]"),.noexpr=tsc("^[-0nNಇ]"),.yesstr=tsc("ಹೌದು"),.nostr=tsc("ಇಲ್ಲ")},.paper={.width=210,.height=297},.telephone={.tel_int_fmt=tsc("+%c ;%a ;%l"),.int_select=tsc("00"),.int_prefix=tsc("91")},.name={.name_fmt=tsc("%p%t%f%t%g"),.name_gen=tsc(""),.name_miss=tsc("Miss."),.name_mr=tsc("Mr."),.name_mrs=tsc("Mrs."),.name_ms=tsc("Ms.")},.address={.postal_fmt=tsc("%z%c%T%s%b%e%r"),.country_name=tsc("ಭಾರತ"),.country_ab2=tsc("IN"),.country_ab3=tsc("IND"),.country_num=356,.country_car=tsc("IND"),.lang_name=tsc("ಕನ್ನಡ"),.lang_ab=tsc("kn"),.lang_term=tsc("kan"),.lang_lib=tsc("kan")},.measurement={.measurement=1}};

inline constexpr wlc_all wlc_all_global{.identification={.name=tsc(L"kn_IN"),.encoding=tsc(FAST_IO_LOCALE_LENCODING),.title=tsc(L"Kannada language locale for India"),.source=tsc(L"IndLinux.org\t\t;\t\tfast_io"),.address=tsc(L"https://gitee.com/qabeowjbtkwb/fast_io\t\t;\t\thttps://github.com/cppfastio/fast_io"),.contact=tsc(L"fast_io"),.email=tsc(L"bug-glibc-locales@gnu.org;euloanty@live.com"),.tel=tsc(L""),.fax=tsc(L""),.language=tsc(L"Kannada"),.territory=tsc(L"India"),.revision=tsc(L"0.1"),.date=tsc(L"2002-11-28")},.monetary={.int_curr_symbol=tsc(L"INR "),.currency_symbol=tsc(L"₹"),.mon_decimal_point=tsc(L"."),.mon_thousands_sep=tsc(L","),.mon_grouping={monetary_mon_grouping_storage,2},.positive_sign=tsc(L""),.negative_sign=tsc(L"-"),.int_frac_digits=2,.frac_digits=2,.p_cs_precedes=1,.p_sep_by_space=0,.n_cs_precedes=1,.n_sep_by_space=0,.p_sign_posn=1,.n_sign_posn=1},.numeric={.decimal_point=tsc(L"."),.thousands_sep=tsc(L","),.grouping={numeric_grouping_storage,1}},.time={.abday={tsc(L"ರ"),tsc(L"ಸೋ"),tsc(L"ಮಂ"),tsc(L"ಬು"),tsc(L"ಗು"),tsc(L"ಶು"),tsc(L"ಶ")},.day={tsc(L"ರವಿವಾರ"),tsc(L"ಸೋಮವಾರ"),tsc(L"ಮಂಗಳವಾರ"),tsc(L"ಬುಧವಾರ"),tsc(L"ಗುರುವಾರ"),tsc(L"ಶುಕ್ರವಾರ"),tsc(L"ಶನಿವಾರ")},.abmon={tsc(L"ಜನ"),tsc(L"ಫೆಬ್ರ"),tsc(L"ಮಾರ್ಚ್"),tsc(L"ಏಪ್ರಿ"),tsc(L"ಮೇ"),tsc(L"ಜೂನ್"),tsc(L"ಜುಲೈ"),tsc(L"ಆ"),tsc(L"ಸೆಪ್ಟೆಂ"),tsc(L"ಅಕ್ಟೋ"),tsc(L"ನವೆಂ"),tsc(L"ಡಿಸೆಂ")},.mon={tsc(L"ಜನವರಿ"),tsc(L"ಫೆಬ್ರವರಿ"),tsc(L"ಮಾರ್ಚ್"),tsc(L"ಏಪ್ರಿಲ್"),tsc(L"ಮೇ"),tsc(L"ಜೂನ್"),tsc(L"ಜುಲೈ"),tsc(L"ಆಗಸ್ಟ್"),tsc(L"ಸೆಪ್ಟೆಂಬರ್"),tsc(L"ಅಕ್ಟೋಬರ್"),tsc(L"ನವೆಂಬರ್"),tsc(L"ಡಿಸೆಂಬರ್")},.d_t_fmt=tsc(L"%A %d %b %Y %I:%M:%S %p"),.d_fmt=tsc(L"%-d//%-m//%y"),.t_fmt=tsc(L"%I:%M:%S %p %Z"),.t_fmt_ampm=tsc(L"%I:%M:%S %p %Z"),.date_fmt=tsc(L"%A %d %b %Y %I:%M:%S %p %Z"),.am_pm={tsc(L"ಪೂರ್ವಾಹ್ನ"),tsc(L"ಅಪರಾಹ್ನ")},.week={7,19971130,1}},.messages={.yesexpr=tsc(L"^[+1yYಹ]"),.noexpr=tsc(L"^[-0nNಇ]"),.yesstr=tsc(L"ಹೌದು"),.nostr=tsc(L"ಇಲ್ಲ")},.paper={.width=210,.height=297},.telephone={.tel_int_fmt=tsc(L"+%c ;%a ;%l"),.int_select=tsc(L"00"),.int_prefix=tsc(L"91")},.name={.name_fmt=tsc(L"%p%t%f%t%g"),.name_gen=tsc(L""),.name_miss=tsc(L"Miss."),.name_mr=tsc(L"Mr."),.name_mrs=tsc(L"Mrs."),.name_ms=tsc(L"Ms.")},.address={.postal_fmt=tsc(L"%z%c%T%s%b%e%r"),.country_name=tsc(L"ಭಾರತ"),.country_ab2=tsc(L"IN"),.country_ab3=tsc(L"IND"),.country_num=356,.country_car=tsc(L"IND"),.lang_name=tsc(L"ಕನ್ನಡ"),.lang_ab=tsc(L"kn"),.lang_term=tsc(L"kan"),.lang_lib=tsc(L"kan")},.measurement={.measurement=1}};

inline constexpr u8lc_all u8lc_all_global{.identification={.name=tsc(u8"kn_IN"),.encoding=tsc(FAST_IO_LOCALE_u8ENCODING),.title=tsc(u8"Kannada language locale for India"),.source=tsc(u8"IndLinux.org\t\t;\t\tfast_io"),.address=tsc(u8"https://gitee.com/qabeowjbtkwb/fast_io\t\t;\t\thttps://github.com/cppfastio/fast_io"),.contact=tsc(u8"fast_io"),.email=tsc(u8"bug-glibc-locales@gnu.org;euloanty@live.com"),.tel=tsc(u8""),.fax=tsc(u8""),.language=tsc(u8"Kannada"),.territory=tsc(u8"India"),.revision=tsc(u8"0.1"),.date=tsc(u8"2002-11-28")},.monetary={.int_curr_symbol=tsc(u8"INR "),.currency_symbol=tsc(u8"₹"),.mon_decimal_point=tsc(u8"."),.mon_thousands_sep=tsc(u8","),.mon_grouping={monetary_mon_grouping_storage,2},.positive_sign=tsc(u8""),.negative_sign=tsc(u8"-"),.int_frac_digits=2,.frac_digits=2,.p_cs_precedes=1,.p_sep_by_space=0,.n_cs_precedes=1,.n_sep_by_space=0,.p_sign_posn=1,.n_sign_posn=1},.numeric={.decimal_point=tsc(u8"."),.thousands_sep=tsc(u8","),.grouping={numeric_grouping_storage,1}},.time={.abday={tsc(u8"ರ"),tsc(u8"ಸೋ"),tsc(u8"ಮಂ"),tsc(u8"ಬು"),tsc(u8"ಗು"),tsc(u8"ಶು"),tsc(u8"ಶ")},.day={tsc(u8"ರವಿವಾರ"),tsc(u8"ಸೋಮವಾರ"),tsc(u8"ಮಂಗಳವಾರ"),tsc(u8"ಬುಧವಾರ"),tsc(u8"ಗುರುವಾರ"),tsc(u8"ಶುಕ್ರವಾರ"),tsc(u8"ಶನಿವಾರ")},.abmon={tsc(u8"ಜನ"),tsc(u8"ಫೆಬ್ರ"),tsc(u8"ಮಾರ್ಚ್"),tsc(u8"ಏಪ್ರಿ"),tsc(u8"ಮೇ"),tsc(u8"ಜೂನ್"),tsc(u8"ಜುಲೈ"),tsc(u8"ಆ"),tsc(u8"ಸೆಪ್ಟೆಂ"),tsc(u8"ಅಕ್ಟೋ"),tsc(u8"ನವೆಂ"),tsc(u8"ಡಿಸೆಂ")},.mon={tsc(u8"ಜನವರಿ"),tsc(u8"ಫೆಬ್ರವರಿ"),tsc(u8"ಮಾರ್ಚ್"),tsc(u8"ಏಪ್ರಿಲ್"),tsc(u8"ಮೇ"),tsc(u8"ಜೂನ್"),tsc(u8"ಜುಲೈ"),tsc(u8"ಆಗಸ್ಟ್"),tsc(u8"ಸೆಪ್ಟೆಂಬರ್"),tsc(u8"ಅಕ್ಟೋಬರ್"),tsc(u8"ನವೆಂಬರ್"),tsc(u8"ಡಿಸೆಂಬರ್")},.d_t_fmt=tsc(u8"%A %d %b %Y %I:%M:%S %p"),.d_fmt=tsc(u8"%-d//%-m//%y"),.t_fmt=tsc(u8"%I:%M:%S %p %Z"),.t_fmt_ampm=tsc(u8"%I:%M:%S %p %Z"),.date_fmt=tsc(u8"%A %d %b %Y %I:%M:%S %p %Z"),.am_pm={tsc(u8"ಪೂರ್ವಾಹ್ನ"),tsc(u8"ಅಪರಾಹ್ನ")},.week={7,19971130,1}},.messages={.yesexpr=tsc(u8"^[+1yYಹ]"),.noexpr=tsc(u8"^[-0nNಇ]"),.yesstr=tsc(u8"ಹೌದು"),.nostr=tsc(u8"ಇಲ್ಲ")},.paper={.width=210,.height=297},.telephone={.tel_int_fmt=tsc(u8"+%c ;%a ;%l"),.int_select=tsc(u8"00"),.int_prefix=tsc(u8"91")},.name={.name_fmt=tsc(u8"%p%t%f%t%g"),.name_gen=tsc(u8""),.name_miss=tsc(u8"Miss."),.name_mr=tsc(u8"Mr."),.name_mrs=tsc(u8"Mrs."),.name_ms=tsc(u8"Ms.")},.address={.postal_fmt=tsc(u8"%z%c%T%s%b%e%r"),.country_name=tsc(u8"ಭಾರತ"),.country_ab2=tsc(u8"IN"),.country_ab3=tsc(u8"IND"),.country_num=356,.country_car=tsc(u8"IND"),.lang_name=tsc(u8"ಕನ್ನಡ"),.lang_ab=tsc(u8"kn"),.lang_term=tsc(u8"kan"),.lang_lib=tsc(u8"kan")},.measurement={.measurement=1}};

inline constexpr u16lc_all u16lc_all_global{.identification={.name=tsc(u"kn_IN"),.encoding=tsc(FAST_IO_LOCALE_uENCODING),.title=tsc(u"Kannada language locale for India"),.source=tsc(u"IndLinux.org\t\t;\t\tfast_io"),.address=tsc(u"https://gitee.com/qabeowjbtkwb/fast_io\t\t;\t\thttps://github.com/cppfastio/fast_io"),.contact=tsc(u"fast_io"),.email=tsc(u"bug-glibc-locales@gnu.org;euloanty@live.com"),.tel=tsc(u""),.fax=tsc(u""),.language=tsc(u"Kannada"),.territory=tsc(u"India"),.revision=tsc(u"0.1"),.date=tsc(u"2002-11-28")},.monetary={.int_curr_symbol=tsc(u"INR "),.currency_symbol=tsc(u"₹"),.mon_decimal_point=tsc(u"."),.mon_thousands_sep=tsc(u","),.mon_grouping={monetary_mon_grouping_storage,2},.positive_sign=tsc(u""),.negative_sign=tsc(u"-"),.int_frac_digits=2,.frac_digits=2,.p_cs_precedes=1,.p_sep_by_space=0,.n_cs_precedes=1,.n_sep_by_space=0,.p_sign_posn=1,.n_sign_posn=1},.numeric={.decimal_point=tsc(u"."),.thousands_sep=tsc(u","),.grouping={numeric_grouping_storage,1}},.time={.abday={tsc(u"ರ"),tsc(u"ಸೋ"),tsc(u"ಮಂ"),tsc(u"ಬು"),tsc(u"ಗು"),tsc(u"ಶು"),tsc(u"ಶ")},.day={tsc(u"ರವಿವಾರ"),tsc(u"ಸೋಮವಾರ"),tsc(u"ಮಂಗಳವಾರ"),tsc(u"ಬುಧವಾರ"),tsc(u"ಗುರುವಾರ"),tsc(u"ಶುಕ್ರವಾರ"),tsc(u"ಶನಿವಾರ")},.abmon={tsc(u"ಜನ"),tsc(u"ಫೆಬ್ರ"),tsc(u"ಮಾರ್ಚ್"),tsc(u"ಏಪ್ರಿ"),tsc(u"ಮೇ"),tsc(u"ಜೂನ್"),tsc(u"ಜುಲೈ"),tsc(u"ಆ"),tsc(u"ಸೆಪ್ಟೆಂ"),tsc(u"ಅಕ್ಟೋ"),tsc(u"ನವೆಂ"),tsc(u"ಡಿಸೆಂ")},.mon={tsc(u"ಜನವರಿ"),tsc(u"ಫೆಬ್ರವರಿ"),tsc(u"ಮಾರ್ಚ್"),tsc(u"ಏಪ್ರಿಲ್"),tsc(u"ಮೇ"),tsc(u"ಜೂನ್"),tsc(u"ಜುಲೈ"),tsc(u"ಆಗಸ್ಟ್"),tsc(u"ಸೆಪ್ಟೆಂಬರ್"),tsc(u"ಅಕ್ಟೋಬರ್"),tsc(u"ನವೆಂಬರ್"),tsc(u"ಡಿಸೆಂಬರ್")},.d_t_fmt=tsc(u"%A %d %b %Y %I:%M:%S %p"),.d_fmt=tsc(u"%-d//%-m//%y"),.t_fmt=tsc(u"%I:%M:%S %p %Z"),.t_fmt_ampm=tsc(u"%I:%M:%S %p %Z"),.date_fmt=tsc(u"%A %d %b %Y %I:%M:%S %p %Z"),.am_pm={tsc(u"ಪೂರ್ವಾಹ್ನ"),tsc(u"ಅಪರಾಹ್ನ")},.week={7,19971130,1}},.messages={.yesexpr=tsc(u"^[+1yYಹ]"),.noexpr=tsc(u"^[-0nNಇ]"),.yesstr=tsc(u"ಹೌದು"),.nostr=tsc(u"ಇಲ್ಲ")},.paper={.width=210,.height=297},.telephone={.tel_int_fmt=tsc(u"+%c ;%a ;%l"),.int_select=tsc(u"00"),.int_prefix=tsc(u"91")},.name={.name_fmt=tsc(u"%p%t%f%t%g"),.name_gen=tsc(u""),.name_miss=tsc(u"Miss."),.name_mr=tsc(u"Mr."),.name_mrs=tsc(u"Mrs."),.name_ms=tsc(u"Ms.")},.address={.postal_fmt=tsc(u"%z%c%T%s%b%e%r"),.country_name=tsc(u"ಭಾರತ"),.country_ab2=tsc(u"IN"),.country_ab3=tsc(u"IND"),.country_num=356,.country_car=tsc(u"IND"),.lang_name=tsc(u"ಕನ್ನಡ"),.lang_ab=tsc(u"kn"),.lang_term=tsc(u"kan"),.lang_lib=tsc(u"kan")},.measurement={.measurement=1}};

inline constexpr u32lc_all u32lc_all_global{.identification={.name=tsc(U"kn_IN"),.encoding=tsc(FAST_IO_LOCALE_UENCODING),.title=tsc(U"Kannada language locale for India"),.source=tsc(U"IndLinux.org\t\t;\t\tfast_io"),.address=tsc(U"https://gitee.com/qabeowjbtkwb/fast_io\t\t;\t\thttps://github.com/cppfastio/fast_io"),.contact=tsc(U"fast_io"),.email=tsc(U"bug-glibc-locales@gnu.org;euloanty@live.com"),.tel=tsc(U""),.fax=tsc(U""),.language=tsc(U"Kannada"),.territory=tsc(U"India"),.revision=tsc(U"0.1"),.date=tsc(U"2002-11-28")},.monetary={.int_curr_symbol=tsc(U"INR "),.currency_symbol=tsc(U"₹"),.mon_decimal_point=tsc(U"."),.mon_thousands_sep=tsc(U","),.mon_grouping={monetary_mon_grouping_storage,2},.positive_sign=tsc(U""),.negative_sign=tsc(U"-"),.int_frac_digits=2,.frac_digits=2,.p_cs_precedes=1,.p_sep_by_space=0,.n_cs_precedes=1,.n_sep_by_space=0,.p_sign_posn=1,.n_sign_posn=1},.numeric={.decimal_point=tsc(U"."),.thousands_sep=tsc(U","),.grouping={numeric_grouping_storage,1}},.time={.abday={tsc(U"ರ"),tsc(U"ಸೋ"),tsc(U"ಮಂ"),tsc(U"ಬು"),tsc(U"ಗು"),tsc(U"ಶು"),tsc(U"ಶ")},.day={tsc(U"ರವಿವಾರ"),tsc(U"ಸೋಮವಾರ"),tsc(U"ಮಂಗಳವಾರ"),tsc(U"ಬುಧವಾರ"),tsc(U"ಗುರುವಾರ"),tsc(U"ಶುಕ್ರವಾರ"),tsc(U"ಶನಿವಾರ")},.abmon={tsc(U"ಜನ"),tsc(U"ಫೆಬ್ರ"),tsc(U"ಮಾರ್ಚ್"),tsc(U"ಏಪ್ರಿ"),tsc(U"ಮೇ"),tsc(U"ಜೂನ್"),tsc(U"ಜುಲೈ"),tsc(U"ಆ"),tsc(U"ಸೆಪ್ಟೆಂ"),tsc(U"ಅಕ್ಟೋ"),tsc(U"ನವೆಂ"),tsc(U"ಡಿಸೆಂ")},.mon={tsc(U"ಜನವರಿ"),tsc(U"ಫೆಬ್ರವರಿ"),tsc(U"ಮಾರ್ಚ್"),tsc(U"ಏಪ್ರಿಲ್"),tsc(U"ಮೇ"),tsc(U"ಜೂನ್"),tsc(U"ಜುಲೈ"),tsc(U"ಆಗಸ್ಟ್"),tsc(U"ಸೆಪ್ಟೆಂಬರ್"),tsc(U"ಅಕ್ಟೋಬರ್"),tsc(U"ನವೆಂಬರ್"),tsc(U"ಡಿಸೆಂಬರ್")},.d_t_fmt=tsc(U"%A %d %b %Y %I:%M:%S %p"),.d_fmt=tsc(U"%-d//%-m//%y"),.t_fmt=tsc(U"%I:%M:%S %p %Z"),.t_fmt_ampm=tsc(U"%I:%M:%S %p %Z"),.date_fmt=tsc(U"%A %d %b %Y %I:%M:%S %p %Z"),.am_pm={tsc(U"ಪೂರ್ವಾಹ್ನ"),tsc(U"ಅಪರಾಹ್ನ")},.week={7,19971130,1}},.messages={.yesexpr=tsc(U"^[+1yYಹ]"),.noexpr=tsc(U"^[-0nNಇ]"),.yesstr=tsc(U"ಹೌದು"),.nostr=tsc(U"ಇಲ್ಲ")},.paper={.width=210,.height=297},.telephone={.tel_int_fmt=tsc(U"+%c ;%a ;%l"),.int_select=tsc(U"00"),.int_prefix=tsc(U"91")},.name={.name_fmt=tsc(U"%p%t%f%t%g"),.name_gen=tsc(U""),.name_miss=tsc(U"Miss."),.name_mr=tsc(U"Mr."),.name_mrs=tsc(U"Mrs."),.name_ms=tsc(U"Ms.")},.address={.postal_fmt=tsc(U"%z%c%T%s%b%e%r"),.country_name=tsc(U"ಭಾರತ"),.country_ab2=tsc(U"IN"),.country_ab3=tsc(U"IND"),.country_num=356,.country_car=tsc(U"IND"),.lang_name=tsc(U"ಕನ್ನಡ"),.lang_ab=tsc(U"kn"),.lang_term=tsc(U"kan"),.lang_lib=tsc(U"kan")},.measurement={.measurement=1}};


}
}

#include"../main.h"